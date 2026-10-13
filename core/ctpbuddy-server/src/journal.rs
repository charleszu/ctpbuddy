//! Append-only JSONL event journal (DESIGN.md §11.4).
//!
//! The single writer is the world loop: events accumulate in memory and are
//! flushed every [`FLUSH_EVENTS`] records or [`FLUSH_EVERY`] of wall time,
//! whichever comes first. An unflushed tail may be lost on a crash; the
//! durability level is chosen with `CTPBUDDY_JOURNAL_SYNC` ([`SyncMode`]).
//! Each startup creates a fresh recording; previous runs are archived separately.

use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::dtime::now_wall;
use crate::json::{self, Value};

const FLUSH_EVENTS: usize = 1000;
const FLUSH_EVERY: std::time::Duration = std::time::Duration::from_millis(100);

/// Env var selecting the journal durability level.
pub const SYNC_ENV: &str = "CTPBUDDY_JOURNAL_SYNC";

/// Env var: `1` stops the world loop after a journal write failure instead of
/// running on with a ledger that is ahead of its authoritative event stream.
pub const HARD_FAIL_ENV: &str = "CTPBUDDY_JOURNAL_HARD_FAIL";

pub fn hard_fail_enabled() -> bool {
    crate::env_flag(std::env::var(HARD_FAIL_ENV).ok().as_deref())
}

/// Read the durable journal tail that belongs to the snapshot sequence.
///
/// `Journal::new` archives the previous run before opening a new writer, so
/// recovery must look in the newest `journal-run-*` directory. A crash can
/// leave an unterminated final line; that line is deliberately ignored while
/// every complete line remains fail-closed.
pub fn read_archived_tail(
    data_dir: &str,
    after_seq: u64,
) -> Result<Vec<crate::json::Value>, String> {
    let root = Path::new(data_dir);
    let mut archives: Vec<PathBuf> = std::fs::read_dir(root)
        .map_err(|e| format!("{}: {e}", root.display()))?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = path.file_name()?.to_str()?;
            (name.starts_with("journal-run-") && path.is_dir()).then_some(path)
        })
        .collect();
    archives.sort();
    let Some(journal_dir) = archives.last().map(|archive| archive.join("journal")) else {
        return if after_seq == 0 {
            Ok(Vec::new())
        } else {
            Err(format!(
                "{}: no archived journal covers snapshot seq {after_seq}",
                root.display()
            ))
        };
    };
    let mut files: Vec<PathBuf> = std::fs::read_dir(&journal_dir)
        .map_err(|e| format!("{}: {e}", journal_dir.display()))?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension().and_then(|ext| ext.to_str()) == Some("jsonl")).then_some(path)
        })
        .collect();
    files.sort();
    let mut parsed = Vec::new();
    for path in files {
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let complete = text.ends_with('\n');
        let lines: Vec<&str> = text.split('\n').collect();
        for (index, line) in lines.iter().enumerate() {
            if line.is_empty() || (!complete && index + 1 == lines.len()) {
                continue;
            }
            let event = crate::json::parse(line)
                .map_err(|e| format!("{}:{}: {e}", path.display(), index + 1))?;
            let seq = event
                .get_num("seq")
                .filter(|value| value.is_finite() && value.fract() == 0.0 && *value >= 0.0)
                .map(|value| value as u64)
                .ok_or_else(|| {
                    format!("{}:{}: seq must be an integer", path.display(), index + 1)
                })?;
            parsed.push((seq, path.clone(), index + 1, event));
        }
    }
    parsed.sort_by_key(|(seq, _, _, _)| *seq);
    let max_seq = parsed.last().map(|(seq, _, _, _)| *seq).unwrap_or(0);
    if max_seq < after_seq {
        return Err(format!(
            "{}: archived journal ends at seq {max_seq}, below snapshot seq {after_seq}",
            root.display()
        ));
    }
    let mut events = Vec::new();
    let mut expected = after_seq + 1;
    for (seq, path, line, event) in parsed {
        if seq <= after_seq {
            continue;
        }
        if seq != expected {
            return Err(format!(
                "{}:{}: journal seq {seq} breaks recovery tail (expected {expected})",
                path.display(),
                line
            ));
        }
        expected += 1;
        events.push(event);
    }
    Ok(events)
}

/// How hard the journal tries to reach stable storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncMode {
    /// flush + fsync after every record: nothing acknowledged is ever lost.
    Always,
    /// flush + fsync every 100 ms / 1000 records (default).
    Interval,
    /// flush to the OS only, never fsync: survives a process crash, not power loss.
    Off,
}

impl SyncMode {
    pub fn parse(v: Option<&str>) -> SyncMode {
        match v.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
            Some("always") => SyncMode::Always,
            Some("off") | Some("none") => SyncMode::Off,
            Some("interval") | Some("") | None => SyncMode::Interval,
            Some(other) => {
                eprintln!("[ctpbuddy] unknown {SYNC_ENV}={other}, using interval");
                SyncMode::Interval
            }
        }
    }
}

pub struct Journal {
    _lock: File,
    dir: PathBuf,
    day: String,
    writer: Option<BufWriter<File>>,
    seq: u64,
    pending: usize,
    last_flush: Instant,
    sync: SyncMode,
    /// Last write/sync failure. The ledger applies an event before it is
    /// journaled, so a silent I/O failure leaves live state ahead of the
    /// authoritative stream; surfacing the error in ADMIN status is the only
    /// real-time signal of that divergence.
    last_error: Option<String>,
    /// Durability operations (record write / flush / fsync) that failed since
    /// the last success. Hard-fail mode stops the world once this is nonzero.
    consecutive_failures: u32,
}

impl Journal {
    /// `dir` is the data directory; journals land in `dir/journal/`.
    pub fn new(dir: &str) -> std::io::Result<Self> {
        create_dir_all(dir)?;
        let root = std::fs::canonicalize(dir)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(root.join(".journal.lock"))?;
        lock.try_lock().map_err(std::io::Error::other)?;
        let jdir = root.join("journal");
        if jdir.exists() {
            if std::fs::symlink_metadata(&jdir)?.file_type().is_symlink()
                || std::fs::canonicalize(&jdir)? != jdir
            {
                return Err(std::io::Error::other(
                    "journal directory must not be a link",
                ));
            }
            let has_events = std::fs::read_dir(&jdir)?.try_fold(false, |found, entry| {
                let entry = entry?;
                Ok::<_, std::io::Error>(
                    found || entry.path().extension().is_some_and(|ext| ext == "jsonl"),
                )
            })?;
            if has_events {
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(std::io::Error::other)?
                    .as_nanos();
                let archive = root.join(format!("journal-run-{stamp}-{}", std::process::id()));
                std::fs::create_dir(&archive)?;
                std::fs::rename(&jdir, archive.join("journal"))?;
            }
        }
        create_dir_all(&jdir)?;
        Ok(Journal {
            _lock: lock,
            dir: jdir,
            day: String::new(),
            writer: None,
            seq: 0,
            pending: 0,
            last_flush: Instant::now(),
            sync: SyncMode::parse(std::env::var(SYNC_ENV).ok().as_deref()),
            last_error: None,
            consecutive_failures: 0,
        })
    }

    fn ensure_writer(&mut self, day: &str) -> std::io::Result<()> {
        if self.writer.is_some() && self.day == day {
            return Ok(());
        }
        self.flush()?;
        let path = self.dir.join(format!("{day}.jsonl"));
        let f = OpenOptions::new().create(true).append(true).open(&path)?;
        // Make the new directory entry durable too (not supported on Windows).
        #[cfg(unix)]
        if self.sync != SyncMode::Off {
            File::open(&self.dir)?.sync_all()?;
        }
        self.writer = Some(BufWriter::new(f));
        self.day = day.to_string();
        Ok(())
    }

    /// Append one event. Line order == world-loop order.
    pub fn record(
        &mut self,
        day: &str,
        vt_ms: f64,
        event_type: &str,
        broker_id: &str,
        investor_id: &str,
        data: Value,
    ) {
        let next_seq = self.seq + 1;
        let iso = now_wall();
        let mut pairs: Vec<(String, Value)> = vec![
            ("seq".into(), json::n(next_seq as f64)),
            ("ts_wall".into(), json::s(&iso)),
            ("trading_day".into(), json::s(day)),
            ("vt_ms".into(), json::n(vt_ms)),
            ("type".into(), json::s(event_type)),
        ];
        if !broker_id.is_empty() {
            pairs.push(("broker".into(), json::s(broker_id)));
        }
        if !investor_id.is_empty() {
            pairs.push(("investor".into(), json::s(investor_id)));
        }
        pairs.push(("data".into(), data));
        let event = Value::Obj(pairs);
        if !crate::journal_contract::validate_event(&event) {
            eprintln!("[ctpbuddy] journal event rejected by contract: {event_type}");
            return;
        }
        let line = event.to_json();
        if let Err(e) = self.ensure_writer(day).and_then(|_| {
            self.writer
                .as_mut()
                .unwrap()
                .write_all(line.as_bytes())
                .and_then(|_| self.writer.as_mut().unwrap().write_all(b"\n"))
        }) {
            self.note_failure("write", &e);
            // Drop the writer; the next record tries to reopen.
            self.writer = None;
            return;
        }
        self.seq = next_seq;
        self.pending += 1;
        self.last_error = None;
        self.consecutive_failures = 0;
        if self.sync == SyncMode::Always {
            if let Err(e) = self.flush() {
                self.note_failure("sync", &e);
            }
        }
    }

    /// Record one failed durability operation (write / flush / fsync): the
    /// ledger has already applied the event, so this divergence must be
    /// visible -- and in hard-fail mode it stops the world.
    fn note_failure(&mut self, context: &str, error: &std::io::Error) {
        eprintln!("[ctpbuddy] journal {context} failed: {error}");
        self.last_error = Some(format!("{context}: {error}"));
        self.consecutive_failures += 1;
    }

    pub fn seq(&self) -> u64 {
        self.seq
    }

    pub fn pending(&self) -> usize {
        self.pending
    }

    /// Last write/sync failure; `None` while the stream is healthy.
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Records that failed to write since the last successful write.
    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }

    /// Test-only: enter the failed-write state without a real I/O failure
    /// (pre-blocking the day file does not work -- `Journal::new` archives any
    /// existing `.jsonl` entry before the first record).
    #[cfg(test)]
    pub(crate) fn simulate_write_failure(&mut self) {
        self.last_error = Some("write: simulated".into());
        self.consecutive_failures += 1;
    }

    /// Flush buffered lines when enough events or time have accumulated.
    /// An idle journal (nothing pending) is never fsynced.
    pub fn flush_if_due(&mut self, now: Instant) {
        if self.pending == 0 {
            return;
        }
        if self.pending >= FLUSH_EVENTS || now.duration_since(self.last_flush) >= FLUSH_EVERY {
            if let Err(error) = self.flush() {
                self.note_failure("flush", &error);
            }
        }
    }

    /// Flush + fsync (call on shutdown / explicit dump).
    pub fn flush(&mut self) -> std::io::Result<()> {
        if let Some(w) = self.writer.as_mut() {
            w.flush()?;
            if self.sync != SyncMode::Off {
                w.get_ref().sync_all()?;
            }
        }
        self.pending = 0;
        self.last_flush = Instant::now();
        if self.consecutive_failures == 0 {
            self.last_error = None;
        }
        Ok(())
    }
}

impl Drop for Journal {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_mode_parsing() {
        assert_eq!(SyncMode::parse(None), SyncMode::Interval);
        assert_eq!(SyncMode::parse(Some("ALWAYS")), SyncMode::Always);
        assert_eq!(SyncMode::parse(Some("off")), SyncMode::Off);
        assert_eq!(SyncMode::parse(Some("bogus")), SyncMode::Interval);
    }

    #[test]
    fn always_mode_makes_every_record_durable_immediately() {
        let dir = std::env::temp_dir().join(format!("ctpbuddy-jsync-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut j = Journal::new(dir.to_str().unwrap()).unwrap();
        j.sync = SyncMode::Always;
        j.record("20261005", 0.0, "server_start", "", "", Value::Null);
        assert_eq!(j.pending(), 0, "flushed without waiting for the interval");
        let file = std::fs::read_to_string(dir.join("journal").join("20261005.jsonl")).unwrap();
        assert_eq!(file.lines().count(), 1);
        drop(j);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_event_is_rejected_without_advancing_sequence() {
        let dir = std::env::temp_dir().join(format!("ctpbuddy-jcontract-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut j = Journal::new(dir.to_str().unwrap()).unwrap();
        j.record("20261005", 0.0, "made_up_event", "", "", Value::Null);
        assert_eq!(j.seq(), 0);
        assert!(!dir.join("journal").join("20261005.jsonl").exists());
        drop(j);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn malformed_envelope_is_rejected_without_advancing_sequence() {
        let dir = std::env::temp_dir().join(format!(
            "ctpbuddy-jcontract-envelope-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let mut j = Journal::new(dir.to_str().unwrap()).unwrap();
        j.record("20261005", f64::NAN, "server_start", "", "", Value::Null);
        j.record("20261005", 0.0, "server_start", "", "", Value::Arr(vec![]));
        j.record(
            "20261005",
            0.0,
            "fill",
            "",
            "",
            Value::Obj(vec![("trade_id".into(), Value::Null)]),
        );
        assert_eq!(j.seq(), 0);
        assert!(!dir.join("journal").join("20261005.jsonl").exists());
        drop(j);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejected_event_does_not_create_a_sequence_gap() {
        let dir = std::env::temp_dir().join(format!(
            "ctpbuddy-jcontract-sequence-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let mut j = Journal::new(dir.to_str().unwrap()).unwrap();
        j.sync = SyncMode::Always;
        j.record("20261005", f64::NAN, "server_start", "", "", Value::Null);
        j.record("20261005", 0.0, "server_start", "", "", Value::Null);
        assert_eq!(j.seq(), 1);
        let file = std::fs::read_to_string(dir.join("journal").join("20261005.jsonl")).unwrap();
        assert_eq!(file.lines().count(), 1);
        assert!(file.lines().next().unwrap().contains("\"seq\":1"));
        drop(j);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn failed_write_does_not_advance_sequence() {
        let dir = std::env::temp_dir().join(format!(
            "ctpbuddy-journal-write-failure-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let mut j = Journal::new(dir.to_str().unwrap()).unwrap();
        std::fs::create_dir(j.dir.join("20261005.jsonl")).unwrap();
        j.record("20261005", 0.0, "server_start", "", "", Value::Null);
        assert_eq!(j.seq(), 0);
        drop(j);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Review R1: a write failure must be observable (ADMIN status reads this
    /// state), and a subsequent successful write must clear it.
    #[test]
    fn write_failure_surfaces_health_and_success_clears_it() {
        let dir = std::env::temp_dir().join(format!("ctpbuddy-jhealth-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut j = Journal::new(dir.to_str().unwrap()).unwrap();
        // block the day file with a directory: every write to it fails
        std::fs::create_dir_all(j.dir.join("20261005.jsonl")).unwrap();
        j.record("20261005", 0.0, "server_start", "", "", Value::Null);
        assert_eq!(j.seq(), 0);
        assert_eq!(j.consecutive_failures(), 1);
        assert!(j.last_error().is_some_and(|e| e.starts_with("write:")));
        // a writable day succeeds and clears the failure state
        j.record("20261006", 0.0, "server_start", "", "", Value::Null);
        assert_eq!(j.seq(), 1);
        assert_eq!(j.consecutive_failures(), 0);
        assert!(j.last_error().is_none());
        drop(j);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn archived_tail_keeps_complete_lines_and_drops_crash_fragment() {
        let dir =
            std::env::temp_dir().join(format!("ctpbuddy-journal-tail-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut first = Journal::new(dir.to_str().unwrap()).unwrap();
        first.sync = SyncMode::Always;
        first.record("20261005", 0.0, "server_start", "", "", Value::Null);
        first.record("20261005", 1.0, "server_stop", "", "", Value::Null);
        drop(first);

        let second = Journal::new(dir.to_str().unwrap()).unwrap();
        drop(second);
        let archive = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let name = path.file_name()?.to_str()?.to_string();
                name.starts_with("journal-run-").then_some(path)
            })
            .next()
            .unwrap();
        let file = archive.join("journal").join("20261005.jsonl");
        let mut text = std::fs::read_to_string(&file).unwrap();
        text.push_str("{\"seq\":3");
        std::fs::write(&file, text).unwrap();

        let tail = read_archived_tail(dir.to_str().unwrap(), 1).unwrap();
        assert_eq!(tail.len(), 1);
        assert_eq!(tail[0].get_num("seq"), Some(2.0));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
