//! Optional WordCraft UI hosted inside the ToolBox egui window.
//! The source file is copied to a temporary directory before editing.
use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use wordcraft_doc::Document;
use wordcraft_engine::Session;
use wordcraft_ui_egui::{Services, WordApp};

pub struct WordCraftEmbed {
    app: WordApp,
    original: PathBuf,
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
        self.app.run("file.save", serde_json::json!({}))?;
        Ok(())
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.app.ui(ui);
    }

    pub fn raw_input_hook(&mut self, raw: &mut egui::RawInput) {
        self.app.raw_input_hook(raw);
    }
}
