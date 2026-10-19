use crate::csv_workspace::{CsvWorkspace, MAX_COLUMNS};
use std::fs;
use tempfile::tempdir;

#[test]
fn utf8_bom_quoted_fields_and_multiline_survive_save() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("한글.csv");
    let original = "\u{feff}이름,설명\r\n홍길동,\"줄1\n줄2,쉼표\"\r\n";
    fs::write(&path, original).unwrap();
    let mut sheet = CsvWorkspace::open(&path).unwrap();
    assert!(sheet.bom);
    assert_eq!(sheet.cell(1, 1), "줄1\n줄2,쉼표");
    sheet.set_cell(1, 0, "김철수".into()).unwrap();
    let backup = sheet.save_current().unwrap().unwrap();
    assert_eq!(fs::read_to_string(backup).unwrap(), original);
    let reloaded = CsvWorkspace::open(&path).unwrap();
    assert_eq!(reloaded.cell(1, 0), "김철수");
    assert_eq!(reloaded.cell(1, 1), "줄1\n줄2,쉼표");
    assert!(fs::read(path).unwrap().starts_with(&[0xEF, 0xBB, 0xBF]));
}

#[test]
fn undo_redo_and_dirty_tracking() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.tsv");
    fs::write(&path, "a\tb\n1\t2\n").unwrap();
    let mut sheet = CsvWorkspace::open(&path).unwrap();
    assert!(!sheet.is_dirty());
    sheet.set_cell(1, 1, "3".into()).unwrap();
    assert!(sheet.is_dirty());
    assert!(sheet.undo());
    assert_eq!(sheet.cell(1, 1), "2");
    assert!(!sheet.is_dirty());
    assert!(sheet.redo());
    assert_eq!(sheet.cell(1, 1), "3");
    sheet.add_row().unwrap();
    assert_eq!(sheet.rows.len(), 3);
    sheet.delete_row(2).unwrap();
    assert_eq!(sheet.rows.len(), 2);
    sheet.add_column().unwrap();
    assert_eq!(sheet.width(), 3);
}

#[test]
fn save_as_does_not_overwrite_existing_files() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.csv");
    let target = dir.path().join("new.tsv");
    fs::write(&path, "a,b\n").unwrap();
    fs::write(&target, "DO NOT OVERWRITE").unwrap();
    let mut sheet = CsvWorkspace::open(&path).unwrap();
    assert!(sheet.save_as(&target).is_err());
    assert_eq!(fs::read_to_string(&target).unwrap(), "DO NOT OVERWRITE");
    fs::remove_file(&target).unwrap();
    sheet.save_as(&target).unwrap();
    assert_eq!(sheet.delimiter, b'\t');
    assert_eq!(CsvWorkspace::open(&target).unwrap().cell(0, 1), "b");
    assert_eq!(fs::read_to_string(path).unwrap(), "a,b\n");
}

#[test]
fn external_modification_blocks_overwrite() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("data.csv");
    fs::write(&path, "a,b\n").unwrap();
    let mut sheet = CsvWorkspace::open(&path).unwrap();
    sheet.set_cell(0, 0, "edited".into()).unwrap();
    fs::write(&path, "external,change\n").unwrap();
    assert!(sheet.save_current().is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), "external,change\n");
}

#[test]
fn rejects_invalid_utf8_and_too_many_columns() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("bad.csv");
    fs::write(&path, [0xFF, b',', b'a']).unwrap();
    assert!(CsvWorkspace::open(&path).is_err());
    fs::write(&path, "x,".repeat(MAX_COLUMNS) + "last\n").unwrap();
    assert!(CsvWorkspace::open(&path).is_err());
}

#[test]
fn empty_file_can_gain_rows_and_be_saved_as() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("empty.csv");
    let target = dir.path().join("filled.csv");
    fs::write(&path, "").unwrap();
    let mut sheet = CsvWorkspace::open(&path).unwrap();
    assert_eq!(sheet.rows.len(), 0);
    sheet.add_row().unwrap();
    sheet.set_cell(0, 0, "첫 셀".into()).unwrap();
    sheet.save_as(&target).unwrap();
    assert_eq!(CsvWorkspace::open(&target).unwrap().cell(0, 0), "첫 셀");
}

#[test]
fn editing_earlier_column_preserves_later_columns_and_ragged_rows() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("ragged.csv");
    fs::write(&path, "one,two,three\nsingle\n").unwrap();
    let mut sheet = CsvWorkspace::open(&path).unwrap();
    sheet.set_cell(0, 0, "changed".into()).unwrap();
    assert_eq!(sheet.cell(0, 1), "two");
    assert_eq!(sheet.cell(0, 2), "three");
    sheet.save_current().unwrap();
    let reloaded = CsvWorkspace::open(&path).unwrap();
    assert_eq!(reloaded.cell(0, 2), "three");
    assert_eq!(reloaded.rows[1].len(), 1);
}
