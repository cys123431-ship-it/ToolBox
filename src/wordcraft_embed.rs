//! Optional WordCraft UI hosted inside the ToolBox egui window.
//! The source file is copied to a temporary directory before editing.
use crate::docx_validation::validate_docx_package;
use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};
use wordcraft_doc::Document;
use wordcraft_engine::Session;
use wordcraft_ui_egui::{Services, WordApp};

pub struct WordCraftEmbed {
    app: WordApp,
    original: PathBuf,
    working: PathBuf,
    _temporary_directory: TempDir,
}

impl WordCraftEmbed {
    pub fn open(path: &Path) -> Result<Self, String> {
        let original = path.canonicalize().map_err(|e| e.to_string())?;
        if !original
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("docx"))
        {
            return Err("현재 DOCX만 지원합니다.".into());
        }
        if fs::metadata(&original).map_err(|e| e.to_string())?.len() > 64 * 1024 * 1024 {
            return Err("64MB 초과 DOCX는 미지원입니다.".into());
        }
        validate_docx_package(&original)?;
        let directory = tempfile::tempdir().map_err(|e| e.to_string())?;
        let copy = directory
            .path()
            .join(original.file_name().ok_or("파일 이름이 없습니다.")?);
        fs::copy(&original, &copy).map_err(|e| e.to_string())?;
        let mut app = WordApp::new(Session::new(Document::new()), Services::default());
        let outcome = app.run(
            "file.open",
            serde_json::json!({"path": copy.to_string_lossy()}),
        )?;
        if outcome.get("pending").is_some() {
            return Err("추가 대화상자가 필요한 DOCX는 지원하지 않습니다.".into());
        }
        Ok(Self {
            app,
            original,
            working: copy,
            _temporary_directory: directory,
        })
    }

    pub fn original(&self) -> &Path {
        &self.original
    }

    pub fn dirty(&self) -> bool {
        self.app.session.dirty
    }

    pub fn save_working_copy(&mut self) -> Result<(), String> {
        if self.app.session.path.as_deref() != Some(self.working.as_path()) {
            return Err("WordCraft에서 다른 파일을 열었거나 저장했습니다.".into());
        }
        let result = self.app.run("file.save", serde_json::json!({}))?;
        if result.get("pending").is_some() {
            return Err("WordCraft 저장이 아직 완료되지 않았습니다.".into());
        }
        validate_docx_package(&self.working)?;
        Ok(())
    }

    pub fn export_as(&mut self) -> Result<Option<PathBuf>, String> {
        let Some(destination) = rfd::FileDialog::new()
            .add_filter("Word 문서", &["docx"])
            .save_file()
        else {
            return Ok(None);
        };
        if destination == self.original {
            return Err("원본을 직접 덮어쓰지 않습니다. 새 파일 이름을 지정하세요.".into());
        }
        self.save_working_copy()?;
        export_new_file(&self.working, &destination)?;
        Ok(Some(destination))
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.app.ui(ui);
    }

    pub fn raw_input_hook(&mut self, raw: &mut egui::RawInput) {
        self.app.raw_input_hook(raw);
    }
}

fn export_new_file(source: &Path, destination: &Path) -> Result<(), String> {
    if !destination
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("docx"))
    {
        return Err("DOCX 확장자로 저장해야 합니다.".into());
    }
    validate_docx_package(source)?;
    let dir = destination.parent().ok_or("저장할 폴더가 없습니다.")?;
    let mut staged = NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    let mut input = fs::File::open(source).map_err(|e| e.to_string())?;
    std::io::copy(&mut input, &mut staged).map_err(|e| e.to_string())?;
    staged.as_file_mut().sync_all().map_err(|e| e.to_string())?;
    staged
        .persist_noclobber(destination)
        .map_err(|e| format!("기존 파일은 덮어쓰지 않습니다: {}", e.error))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_is_non_destructive() {
        let d = tempfile::tempdir().unwrap();
        let source = d.path().join("working.docx");
        let dest = d.path().join("export.docx");
        crate::docx_validation::write_minimal_docx(&source);
        export_new_file(&source, &dest).unwrap();
        assert_eq!(fs::read(&dest).unwrap(), fs::read(&source).unwrap());
        fs::write(&dest, b"existing").unwrap();
        assert!(export_new_file(&source, &dest).is_err());
        assert_eq!(fs::read(&dest).unwrap(), b"existing");
    }

    #[test]
    fn export_rejects_invalid_docx_without_creating_destination() {
        let d = tempfile::tempdir().unwrap();
        let source = d.path().join("broken.docx");
        let destination = d.path().join("should-not-exist.docx");
        fs::write(&source, b"not a DOCX archive").unwrap();
        assert!(export_new_file(&source, &destination).is_err());
        assert!(!destination.exists());
    }
}
