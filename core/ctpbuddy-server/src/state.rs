//! Crash-recovery snapshot of the ledger (`<data_dir>/state/ledger.json`).
//!
//! The journal is the audit/replay log; this file is the fast restart path.
//! It is written atomically (temp file + fsync + rename + directory fsync) on
//! settlement, shutdown and periodically while the ledger changes, and is read
//! only when recovery is requested (`--recover` / `CTPBUDDY_RECOVER=1`), so a
//! plain start keeps its fresh-world semantics. See `Ledger::to_snapshot` for
//! what is and is not captured (working orders are not).

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use ctpbuddy_ledger::Ledger;
use serde_json::{json, Value};

pub const RECOVER_ENV: &str = "CTPBUDDY_RECOVER";
const FILE_VERSION: u64 = 1;

pub fn state_path(data_dir: &str) -> PathBuf {
    Path::new(data_dir).join("state").join("ledger.json")
}

pub fn recover_requested(env: Option<&str>) -> bool {
    matches!(env.map(str::trim), Some("1") | Some("true"))
}

/// Atomically replace the snapshot file.
pub fn write(data_dir: &str, trading_day: &str, snapshot: &Value) -> std::io::Result<()> {
    let path = state_path(data_dir);
    let dir = path.parent().expect("state path has a parent");
    fs::create_dir_all(dir)?;
    let doc = json!({
        "file_version": FILE_VERSION,
        "trading_day": trading_day,
        "ledger": snapshot,
    });
    let tmp = dir.join("ledger.json.tmp");
    {
        let mut f = File::create(&tmp)?;
        f.write_all(doc.to_string().as_bytes())?;
        f.sync_all()?;
    }
    fs::rename(&tmp, &path)?;
    // Persist the rename itself; opening a directory is not supported on Windows.
    #[cfg(unix)]
    File::open(dir)?.sync_all()?;
    Ok(())
}

/// `Ok(None)` when no snapshot exists; `Err` when one exists but is unusable
/// (recovery must not silently fall back to an empty book).
pub fn read(data_dir: &str) -> Result<Option<(String, Ledger)>, String> {
    let path = state_path(data_dir);
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    let doc: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if doc.get("file_version").and_then(Value::as_u64) != Some(FILE_VERSION) {
        return Err(format!("{}: unsupported file_version", path.display()));
    }
    let day = doc
        .get("trading_day")
        .and_then(Value::as_str)
        .ok_or("snapshot: missing trading_day")?
        .to_string();
    let ledger = Ledger::from_snapshot_with_trading_day(
        doc.get("ledger").ok_or("snapshot: missing ledger")?,
        Some(&day),
    )?;
    Ok(Some((day, ledger)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(tag: &str) -> String {
        let d = std::env::temp_dir().join(format!("ctpbuddy-state-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d.to_string_lossy().into_owned()
    }

    #[test]
    fn write_read_roundtrip_and_missing_file() {
        let dir = tmpdir("rt");
        assert!(read(&dir).unwrap().is_none());
        let mut l = Ledger::new(1234.5);
        l.ensure_account("8888", "alice");
        write(&dir, "20261005", &l.to_snapshot()).unwrap();
        let (day, back) = read(&dir).unwrap().unwrap();
        assert_eq!(day, "20261005");
        assert_eq!(back.to_snapshot(), l.to_snapshot());
        // overwrite is atomic: no temp file left behind
        write(&dir, "20261006", &l.to_snapshot()).unwrap();
        assert!(!state_path(&dir).with_file_name("ledger.json.tmp").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_migrates_v1_snapshot_and_rebuilds_today_fee_pool() {
        let dir = tmpdir("v1-migrate");
        let mut ledger = Ledger::new(1234.5);
        ledger.ensure_account("8888", "alice");
        let position = ledger.position_mut_or_create(
            "8888",
            "alice",
            "TEST",
            ctpbuddy_ledger::PositionSide::Long,
        );
        position.today_position = 3;
        position.open_volume = 3;
        position.add_detail(
            "20261005",
            "T1",
            10.0,
            3,
            ctpbuddy_ledger::Money::ZERO,
            10.0,
        );

        let mut legacy = ledger.to_snapshot();
        legacy["version"] = json!(1);
        for position in legacy["positions"].as_array_mut().unwrap() {
            position.as_object_mut().unwrap().remove("fee_open_pool");
        }
        write(&dir, "20261005", &legacy).unwrap();
        let (day, back) = read(&dir).unwrap().unwrap();
        assert_eq!(day, "20261005");
        assert_eq!(
            back.position("8888", "alice", "TEST", ctpbuddy_ledger::PositionSide::Long)
                .unwrap()
                .fee_open_pool,
            3
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_snapshot_is_a_hard_error() {
        let dir = tmpdir("bad");
        fs::create_dir_all(state_path(&dir).parent().unwrap()).unwrap();
        fs::write(state_path(&dir), b"{ not json").unwrap();
        assert!(read(&dir).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn recover_flag_parsing() {
        assert!(recover_requested(Some("1")));
        assert!(recover_requested(Some("true")));
        assert!(!recover_requested(Some("0")));
        assert!(!recover_requested(None));
    }
}
