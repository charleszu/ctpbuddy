//! Append-only JSONL event journal (DESIGN.md §11.4).
//!
//! The single writer is the world loop: events accumulate in memory and are
//! flushed every [`FLUSH_EVENTS`] records or [`FLUSH_EVERY`] of wall time,
//! whichever comes first. An unflushed tail may be lost on a crash.
//! Each startup creates a fresh recording; previous runs are archived separately.

use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use crate::dtime::now_wall;
use crate::json::{self, Value};

const FLUSH_EVENTS: usize = 1000;
const FLUSH_EVERY: std::time::Duration = std::time::Duration::from_millis(100);

pub struct Journal {
    _lock: File,
    dir: PathBuf,
    day: String,
    writer: Option<BufWriter<File>>,
    seq: u64,
    pending: usize,
    last_flush: Instant,
}

impl Journal {
    /// `dir` is the data directory; journals land in `dir/journal/`.
    pub fn new(dir: &str) -> std::io::Result<Self> {
        create_dir_all(dir)?;
        let root = std::fs::canonicalize(dir)?;
        let lock = OpenOptions::new().create(true).truncate(false).read(true).write(true)
            .open(root.join(".journal.lock"))?;
        lock.try_lock().map_err(std::io::Error::other)?;
        let jdir = root.join("journal");
        if jdir.exists() {
            if std::fs::symlink_metadata(&jdir)?.file_type().is_symlink()
                || std::fs::canonicalize(&jdir)? != jdir {
                return Err(std::io::Error::other("journal directory must not be a link"));
            }
            let has_events = std::fs::read_dir(&jdir)?.try_fold(false, |found, entry| {
                let entry = entry?;
                Ok::<_, std::io::Error>(found || entry.path().extension().is_some_and(|ext| ext == "jsonl"))
            })?;
            if has_events {
                let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                    .map_err(std::io::Error::other)?.as_nanos();
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
        })
    }

    fn ensure_writer(&mut self, day: &str) -> std::io::Result<()> {
        if self.writer.is_some() && self.day == day {
            return Ok(());
        }
        self.flush()?;
        let path = self.dir.join(format!("{day}.jsonl"));
        let f = OpenOptions::new().create(true).append(true).open(&path)?;
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
    }

    pub fn seq(&self) -> u64 {
        self.seq
    }

    pub fn pending(&self) -> usize {
        self.pending
    }

    /// Flush buffered lines when enough events or time have accumulated.
    pub fn flush_if_due(&mut self, now: Instant) {
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
            w.get_ref().sync_all()?;
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
