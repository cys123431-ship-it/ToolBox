//! Bounded, local CSV/TSV editor. This is not an XLSX engine.
use csv::{ReaderBuilder, Terminator, WriterBuilder};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::{Builder, NamedTempFile};

pub const MAX_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_ROWS: usize = 2_000;
pub const MAX_COLUMNS: usize = 48;
const MAX_UNDO: usize = 32;

#[derive(Debug)]
pub struct CsvWorkspace {
    pub path: PathBuf,
    pub rows: Vec<Vec<String>>,
    pub delimiter: u8,
    pub bom: bool,
    saved_rows: Vec<Vec<String>>,
    source_bytes: Vec<u8>,
    undo: Vec<Vec<Vec<String>>>,
    redo: Vec<Vec<Vec<String>>>,
}

fn delimiter_for(path: &Path) -> Result<u8, String> {
    match path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "csv" => Ok(b','),
        "tsv" => Ok(b'\t'),
        _ => Err("CSV/TSV 파일만 지원합니다.".into()),
    }
}

fn check_regular_file(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err("심볼릭 링크가 아닌 일반 파일만 지원합니다.".into());
    }
    Ok(())
}

impl CsvWorkspace {
    pub fn open(path: &Path) -> Result<Self, String> {
        let delimiter = delimiter_for(path)?;
        check_regular_file(path)?;
        if fs::metadata(path).map_err(|e| e.to_string())?.len() > MAX_BYTES as u64 {
            return Err("2MB 초과 CSV/TSV 파일은 지원하지 않습니다.".into());
        }
        let source_bytes = fs::read(path).map_err(|e| e.to_string())?;
        let bom = source_bytes.starts_with(&[0xEF, 0xBB, 0xBF]);
        let data = if bom {
            &source_bytes[3..]
        } else {
            &source_bytes[..]
        };
        let mut reader = ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .delimiter(delimiter)
            .from_reader(data);
        let mut rows = Vec::new();
        for result in reader.records() {
            let record = result.map_err(|e| format!("CSV/TSV 구문 또는 UTF-8 오류: {e}"))?;
            if record.len() > MAX_COLUMNS {
                return Err(format!("최대 {MAX_COLUMNS}열까지만 지원합니다."));
            }
            rows.push(record.iter().map(str::to_owned).collect());
            if rows.len() > MAX_ROWS {
                return Err(format!("최대 {MAX_ROWS}행까지만 지원합니다."));
            }
        }
        Ok(Self {
            path: path.to_path_buf(),
            saved_rows: rows.clone(),
            rows,
            delimiter,
            bom,
            source_bytes,
            undo: Vec::new(),
            redo: Vec::new(),
        })
    }

    pub fn is_dirty(&self) -> bool {
        self.rows != self.saved_rows
    }
    pub fn width(&self) -> usize {
        self.rows.iter().map(Vec::len).max().unwrap_or(1).max(1)
    }
    pub fn cell(&self, row: usize, col: usize) -> &str {
        self.rows
            .get(row)
            .and_then(|r| r.get(col))
            .map(String::as_str)
            .unwrap_or("")
    }
    fn snapshot(&mut self) {
        if self.undo.len() == MAX_UNDO {
            self.undo.remove(0);
        }
        self.undo.push(self.rows.clone());
        self.redo.clear();
    }
    pub fn set_cell(&mut self, row: usize, col: usize, value: String) -> Result<bool, String> {
        if row >= self.rows.len() || col >= MAX_COLUMNS {
            return Err("유효하지 않은 셀 위치입니다.".into());
        }
        if self.cell(row, col) == value {
            return Ok(false);
        }
        self.snapshot();
        let cells = &mut self.rows[row];
        if col >= cells.len() {
            cells.resize(col + 1, String::new());
        }
        cells[col] = value;
        Ok(true)
    }
    pub fn add_row(&mut self) -> Result<(), String> {
        if self.rows.len() >= MAX_ROWS {
            return Err(format!("최대 {MAX_ROWS}행까지만 지원합니다."));
        }
        let width = self.width();
        self.snapshot();
        self.rows.push(vec![String::new(); width]);
        Ok(())
    }
    pub fn delete_row(&mut self, row: usize) -> Result<(), String> {
        if row >= self.rows.len() {
            return Err("삭제할 행이 없습니다.".into());
        }
        self.snapshot();
        self.rows.remove(row);
        Ok(())
    }
    pub fn add_column(&mut self) -> Result<(), String> {
        if self.rows.is_empty() {
            return Err("먼저 행을 추가하세요.".into());
        }
        if self.width() >= MAX_COLUMNS {
            return Err(format!("최대 {MAX_COLUMNS}열까지만 지원합니다."));
        }
        let width = self.width();
        self.snapshot();
        for row in &mut self.rows {
            row.resize(width + 1, String::new());
        }
        Ok(())
    }
    pub fn undo(&mut self) -> bool {
        if let Some(previous) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.rows, previous));
            true
        } else {
            false
        }
    }
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.rows, next));
            true
        } else {
            false
        }
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    fn encode(&self, delimiter: u8) -> Result<Vec<u8>, String> {
        let mut writer = WriterBuilder::new()
            .has_headers(false)
            .flexible(true)
            .delimiter(delimiter)
            .terminator(Terminator::Any(b'\n'))
            .from_writer(Vec::new());
        for row in &self.rows {
            writer.write_record(row.iter()).map_err(|e| e.to_string())?;
        }
        let data = writer
            .into_inner()
            .map_err(|e| e.into_error().to_string())?;
        let mut bytes = if self.bom {
            vec![0xEF, 0xBB, 0xBF]
        } else {
            Vec::new()
        };
        bytes.extend(data);
        if bytes.len() > MAX_BYTES {
            return Err("저장 파일이 2MB를 초과합니다.".into());
        }
        Ok(bytes)
    }
    fn temporary_file(parent: &Path, bytes: &[u8]) -> Result<NamedTempFile, String> {
        let mut temp = NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        temp.write_all(bytes).map_err(|e| e.to_string())?;
        temp.as_file().sync_all().map_err(|e| e.to_string())?;
        Ok(temp)
    }
    /// Original is backed up before replacement. Reject external modifications.
    pub fn save_current(&mut self) -> Result<Option<PathBuf>, String> {
        if !self.is_dirty() {
            return Ok(None);
        }
        check_regular_file(&self.path)?;
        if fs::read(&self.path).map_err(|e| e.to_string())? != self.source_bytes {
            return Err(
                "외부에서 변경된 파일입니다. 덮어쓰지 않고 다른 이름으로 저장하세요.".into(),
            );
        }
        let bytes = self.encode(self.delimiter)?;
        let parent = self.path.parent().ok_or("저장 폴더가 없습니다.")?;
        let temporary = Self::temporary_file(parent, &bytes)?;
        let backup = Builder::new()
            .prefix(".toolbox-")
            .suffix(".bak")
            .tempfile_in(parent)
            .map_err(|e| e.to_string())?;
        fs::copy(&self.path, backup.path()).map_err(|e| e.to_string())?;
        backup.as_file().sync_all().map_err(|e| e.to_string())?;
        let (file, backup_path) = backup.keep().map_err(|e| e.error.to_string())?;
        drop(file);
        temporary
            .persist(&self.path)
            .map_err(|e| e.error.to_string())?;
        self.source_bytes = bytes;
        self.saved_rows = self.rows.clone();
        Ok(Some(backup_path))
    }
    /// Never overwrites an existing destination.
    pub fn save_as(&mut self, target: &Path) -> Result<(), String> {
        let delimiter = delimiter_for(target)?;
        if target == self.path {
            return Err("현재 파일은 저장 버튼을 사용하세요.".into());
        }
        if target.exists() {
            return Err("대상 파일이 이미 존재합니다.".into());
        }
        let bytes = self.encode(delimiter)?;
        let parent = target.parent().ok_or("저장 폴더가 없습니다.")?;
        Self::temporary_file(parent, &bytes)?
            .persist_noclobber(target)
            .map_err(|e| e.error.to_string())?;
        self.path = target.to_path_buf();
        self.delimiter = delimiter;
        self.source_bytes = bytes;
        self.saved_rows = self.rows.clone();
        Ok(())
    }
}
