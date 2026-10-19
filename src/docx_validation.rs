//! Conservative ZIP-level validation for the WordCraft DOCX preview.
//! This is NOT an OOXML compatibility or fidelity guarantee.
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use zip::ZipArchive;

const MAX_PARTS: usize = 4_096;
const MAX_PART_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 128 * 1024 * 1024;
const REQUIRED_PARTS: [&str; 3] = [
    "[Content_Types].xml",
    "_rels/.rels",
    "word/document.xml",
];

/// Verify that a candidate DOCX is a readable, bounded ZIP package containing
/// its required main parts. Every part is read to EOF to check ZIP CRC values.
/// This does not validate XML, relationships, fonts, layout, or Word fidelity.
pub fn validate_docx_package(path: &Path) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("DOCX 열기 실패: {e}"))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("DOCX ZIP 손상: {e}"))?;
    if archive.len() > MAX_PARTS {
        return Err("DOCX의 ZIP 항목 수가 안전 제한을 초과합니다.".into());
    }

    let mut seen = HashSet::new();
    let mut required = [false; REQUIRED_PARTS.len()];
    let mut total = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("DOCX ZIP 항목 읽기 실패: {e}"))?;
        let name = entry.name().to_owned();
        if name.starts_with('/')
            || name.contains('\\')
            || name.contains(':')
            || name.split('/').any(|part| part == "." || part == "..")
        {
            return Err(format!("DOCX에 허용되지 않는 ZIP 경로가 있습니다: {name}"));
        }
        if !seen.insert(name.clone()) {
            return Err(format!("DOCX에 중복된 ZIP 항목이 있습니다: {name}"));
        }
        if entry.is_dir() {
            continue;
        }
        if entry.size() > MAX_PART_BYTES {
            return Err(format!("DOCX 항목 크기 제한 초과: {name}"));
        }
        total = total
            .checked_add(entry.size())
            .ok_or("DOCX 압축 해제 크기가 너무 큽니다.")?;
        if total > MAX_TOTAL_BYTES {
            return Err("DOCX 전체 압축 해제 크기 제한을 초과합니다.".into());
        }
        let actual = io::copy(
            &mut Read::take(&mut entry, MAX_PART_BYTES + 1),
            &mut io::sink(),
        )
        .map_err(|e| format!("DOCX 항목 손상 또는 CRC 오류 ({name}): {e}"))?;
        if actual != entry.size() {
            return Err(format!("DOCX 항목 크기 불일치: {name}"));
        }
        for (found, expected) in required.iter_mut().zip(REQUIRED_PARTS) {
            if name == expected {
                *found = true;
            }
        }
    }
    for (found, name) in required.into_iter().zip(REQUIRED_PARTS) {
        if !found {
            return Err(format!("DOCX 필수 구성 요소가 없습니다: {name}"));
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn write_minimal_docx(path: &Path) {
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    let file = File::create(path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    let parts = [
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>한글 테스트</w:t></w:r></w:p></w:body></w:document>"#,
        ),
    ];
    for (name, xml) in parts {
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(xml.as_bytes()).unwrap();
    }
    writer.finish().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    #[test]
    fn minimal_korean_docx_package_is_readable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("한글 문서.docx");
        write_minimal_docx(&path);
        validate_docx_package(&path).unwrap();
    }

    #[test]
    fn invalid_zip_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.docx");
        std::fs::write(&path, b"PK but not a zip").unwrap();
        assert!(validate_docx_package(&path).is_err());
    }

    #[test]
    fn missing_main_document_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.docx");
        let mut writer = zip::ZipWriter::new(File::create(&path).unwrap());
        writer
            .start_file("[Content_Types].xml", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"<Types/>").unwrap();
        writer.finish().unwrap();
        assert!(validate_docx_package(&path)
            .unwrap_err()
            .contains("필수 구성 요소"));
    }

    #[test]
    fn traversal_path_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("traversal.docx");
        let mut writer = zip::ZipWriter::new(File::create(&path).unwrap());
        writer
            .start_file("../malicious.xml", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"<malicious/>").unwrap();
        writer.finish().unwrap();
        assert!(validate_docx_package(&path)
            .unwrap_err()
            .contains("허용되지 않는 ZIP 경로"));
    }

    #[test]
    fn duplicate_parts_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("duplicate.docx");
        let mut writer = zip::ZipWriter::new(File::create(&path).unwrap());
        for _ in 0..2 {
            writer
                .start_file("word/document.xml", SimpleFileOptions::default())
                .unwrap();
            writer.write_all(b"<w:document/>").unwrap();
        }
        writer.finish().unwrap();
        assert!(validate_docx_package(&path)
            .unwrap_err()
            .contains("중복된 ZIP 항목"));
    }
}
