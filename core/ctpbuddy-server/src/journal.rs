//! Append-only JSONL event journal (DESIGN.md §11.4).
//!
//! The single writer is the world loop: events accumulate in memory and are
//! flushed every [`FLUSH_EVENTS`] records or [`FLUSH_EVERY`] of wall time,
//! whichever comes first. An unflushed tail may be lost on a crash; the
//! durability level is chosen with `CTPBUDDY_JOURNAL_SYNC` ([`SyncMode`]).
//! Each startup creates a fresh recording; previous runs are archived separately.

use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use crate::dtime::now_wall;
use crate::json::{self, Value};

const FLUSH_EVENTS: usize = 1000;
const FLUSH_EVERY: std::time::Duration = std::time::Duration::from_millis(100);

/// Env var selecting the journal durability level.
pub const SYNC_ENV: &str = "CTPBUDDY_JOURNAL_SYNC";

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
        self.seq += 1;
        let (_d, _t, _ms, iso) = now_wall();
        let mut pairs: Vec<(String, Value)> = vec![
            ("seq".into(), json::n(self.seq as f64)),
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
        let line = Value::Obj(pairs).to_json();
        if let Err(e) = self.ensure_writer(day).and_then(|_| {
            self.writer
                .as_mut()
                .unwrap()
                .write_all(line.as_bytes())
                .and_then(|_| self.writer.as_mut().unwrap().write_all(b"\n"))
        }) {
            eprintln!("[ctpbuddy] journal write failed: {e}");
            // Drop the writer; the next record tries to reopen.
            self.writer = None;
            return;
        }
        self.pending += 1;
        if self.sync == SyncMode::Always {
            if let Err(e) = self.flush() {
                eprintln!("[ctpbuddy] journal sync failed: {e}");
            }
        }
    }

    pub fn seq(&self) -> u64 {
        self.seq
    }

    pub fn pending(&self) -> usize {
        self.pending
    }

    /// Flush buffered lines when enough events or time have accumulated.
    /// An idle journal (nothing pending) is never fsynced.
    pub fn flush_if_due(&mut self, now: Instant) {
        if self.pending == 0 {
            return;
        }
        if self.pending >= FLUSH_EVENTS || now.duration_since(self.last_flush) >= FLUSH_EVERY {
            if let Err(error) = self.flush() {
                eprintln!("[ctpbuddy] journal flush failed: {error}");
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
        j.record("20261005", 0.0, "x", "", "", Value::Null);
        assert_eq!(j.pending(), 0, "flushed without waiting for the interval");
        let file = std::fs::read_to_string(dir.join("journal").join("20261005.jsonl")).unwrap();
        assert_eq!(file.lines().count(), 1);
        drop(j);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
