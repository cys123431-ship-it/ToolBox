//! Durable, local recovery snapshots for CSV/TSV edits.
//! Snapshots are never applied automatically and never overwrite the source file.
use crate::csv_workspace::{MAX_BYTES, MAX_COLUMNS, MAX_ROWS};
use crate::preferences::preferences_path;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

const VERSION: u8 = 1;
const MAX_SNAPSHOT_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoverySnapshot {
    pub version: u8,
    pub source_path: PathBuf,
    pub source_sha256: String,
    pub rows: Vec<Vec<String>>,
    pub selected: Option<(usize, usize)>,
    pub draft: String,
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn stable_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

pub fn snapshot_path(source: &Path) -> PathBuf {
    let identifier = digest(stable_path(source).to_string_lossy().as_bytes());
    preferences_path()
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("recovery")
        .join(format!("{identifier}.json"))
}

impl RecoverySnapshot {
    pub fn new(
        source_path: &Path,
        source_sha256: String,
        rows: Vec<Vec<String>>,
        selected: Option<(usize, usize)>,
        draft: String,
    ) -> Self {
        Self {
            version: VERSION,
            source_path: stable_path(source_path),
            source_sha256,
            rows,
            selected,
            draft,
        }
    }

    fn validate(&self, expected_source: &Path) -> Result<(), String> {
        if self.version != VERSION {
            return Err("지원하지 않는 CSV 복구 데이터 버전입니다.".into());
        }
        if self.source_path != stable_path(expected_source) {
            return Err("다른 파일의 CSV 복구 데이터입니다.".into());
        }
        if self.source_sha256.len() != 64
            || !self.source_sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("복구 데이터의 원본 확인값이 올바르지 않습니다.".into());
        }
        if self.rows.len() > MAX_ROWS
            || self.rows.iter().any(|row| row.len() > MAX_COLUMNS)
            || self.draft.len() > MAX_BYTES
        {
            return Err("CSV 복구 데이터가 편집기 한도를 초과합니다.".into());
        }
        let total = self
            .rows
            .iter()
            .flat_map(|row| row.iter())
            .try_fold(0usize, |size, cell| size.checked_add(cell.len()))
            .ok_or("CSV 복구 데이터 크기가 잘못되었습니다.")?;
        if total > MAX_SNAPSHOT_BYTES as usize {
            return Err("CSV 복구 데이터가 너무 큽니다.".into());
        }
        if let Some((row, col)) = self.selected {
            if row >= self.rows.len() || col >= MAX_COLUMNS {
                return Err("CSV 복구 데이터의 선택 셀이 잘못되었습니다.".into());
            }
        }
        Ok(())
    }
}

fn write_at(destination: &Path, snapshot: &RecoverySnapshot) -> Result<(), String> {
    snapshot.validate(&snapshot.source_path)?;
    let content = serde_json::to_vec(snapshot).map_err(|e| e.to_string())?;
    if content.len() as u64 > MAX_SNAPSHOT_BYTES {
        return Err("복구 데이터가 8MB를 초과합니다.".into());
    }
    let parent = destination.parent().ok_or("복구 폴더가 없습니다.")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut temp = NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temp.write_all(&content).map_err(|e| e.to_string())?;
    temp.as_file().sync_all().map_err(|e| e.to_string())?;
    temp.persist(destination).map_err(|e| e.error.to_string())?;
    Ok(())
}

fn read_at(location: &Path, source: &Path) -> Result<Option<RecoverySnapshot>, String> {
    let metadata = match fs::symlink_metadata(location) {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("CSV 복구 데이터가 일반 파일이 아닙니다.".into());
    }
    if metadata.len() > MAX_SNAPSHOT_BYTES {
        return Err("CSV 복구 데이터가 8MB를 초과합니다.".into());
    }
    let bytes = fs::read(location).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_SNAPSHOT_BYTES {
        return Err("CSV 복구 데이터가 8MB를 초과합니다.".into());
    }
    let snapshot: RecoverySnapshot =
        serde_json::from_slice(&bytes).map_err(|e| format!("복구 데이터 오류: {e}"))?;
    snapshot.validate(source)?;
    Ok(Some(snapshot))
}

pub fn save(snapshot: &RecoverySnapshot) -> Result<(), String> {
    write_at(&snapshot_path(&snapshot.source_path), snapshot)
}

pub fn load(source: &Path) -> Result<Option<RecoverySnapshot>, String> {
    read_at(&snapshot_path(source), source)
}

pub fn discard(source: &Path) -> Result<(), String> {
    match fs::remove_file(snapshot_path(source)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn snapshots_roundtrip_and_do_not_touch_source() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("한글.csv");
        let snapshot_file = dir.path().join("recovery.json");
        fs::write(&source, "a,b\n").unwrap();
        let snap = RecoverySnapshot::new(
            &source,
            digest(b"a,b\n"),
            vec![vec!["한글".into(), "줄\n바꿈".into()]],
            Some((0, 0)),
            "임시 입력".into(),
        );
        write_at(&snapshot_file, &snap).unwrap();
        assert_eq!(read_at(&snapshot_file, &source).unwrap(), Some(snap));
        assert_eq!(fs::read_to_string(&source).unwrap(), "a,b\n");
    }

    #[test]
    fn snapshot_rejects_wrong_source_and_corruption() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("a.csv");
        let other = dir.path().join("b.csv");
        let snapshot_file = dir.path().join("recovery.json");
        fs::write(&source, "a\n").unwrap();
        fs::write(&other, "b\n").unwrap();
        let snap = RecoverySnapshot::new(
            &source,
            digest(b"a\n"),
            vec![vec!["changed".into()]],
            Some((0, 0)),
            String::new(),
        );
        write_at(&snapshot_file, &snap).unwrap();
        assert!(read_at(&snapshot_file, &other).is_err());
        fs::write(&snapshot_file, "{broken").unwrap();
        assert!(read_at(&snapshot_file, &source).is_err());
    }

    #[test]
    fn snapshot_rejects_oversized_rows_and_invalid_cell() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("a.csv");
        fs::write(&source, "").unwrap();
        let mut snap = RecoverySnapshot::new(
            &source,
            digest(b""),
            vec![vec!["ok".into()]],
            Some((0, MAX_COLUMNS)),
            String::new(),
        );
        assert!(snap.validate(&source).is_err());
        snap.selected = None;
        snap.rows = vec![vec!["x".into(); MAX_COLUMNS + 1]];
        assert!(snap.validate(&source).is_err());
    }

    #[test]
    fn snapshot_location_is_deterministic_per_file() {
        let dir = tempdir().unwrap();
        let first = dir.path().join("one.csv");
        let second = dir.path().join("two.csv");
        fs::write(&first, "").unwrap();
        fs::write(&second, "").unwrap();
        assert_eq!(snapshot_path(&first), snapshot_path(&first));
        assert_ne!(snapshot_path(&first), snapshot_path(&second));
    }
}
