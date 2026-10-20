//! Owns one live editor instance per open document. Switching tabs never drops edits.
//! External-only tabs are file references, not in-process editors.
use crate::csv_ui::CsvEditorUi;
#[cfg(feature = "embedded-wordcraft")]
use crate::wordcraft_embed::WordCraftEmbed;
use crate::workspace::{AdapterKind, FormatKind};
use std::path::{Path, PathBuf};

enum EditorSession {
    Csv(CsvEditorUi),
    #[cfg(feature = "embedded-wordcraft")]
    WordCraft(WordCraftEmbed),
    ExternalOnly,
}

pub struct DocumentSession {
    pub path: PathBuf,
    pub format: FormatKind,
    editor: EditorSession,
}

impl DocumentSession {
    pub fn adapter(&self) -> AdapterKind {
        match self.editor {
            EditorSession::Csv(_) => AdapterKind::NativeCsv,
            #[cfg(feature = "embedded-wordcraft")]
            EditorSession::WordCraft(_) => AdapterKind::ExperimentalWordCraft,
            EditorSession::ExternalOnly => self.format.adapter(),
        }
    }
}

#[derive(Default)]
pub struct DocumentSessions {
    sessions: Vec<DocumentSession>,
    active: Option<usize>,
}

impl DocumentSessions {
    pub fn sessions(&self) -> &[DocumentSession] {
        &self.sessions
    }
    pub fn active_index(&self) -> Option<usize> {
        self.active
    }
    pub fn active(&self) -> Option<&DocumentSession> {
        self.active.and_then(|index| self.sessions.get(index))
    }
    pub fn active_path(&self) -> Option<&Path> {
        self.active().map(|session| session.path.as_path())
    }
    #[cfg(feature = "embedded-wordcraft")]
    pub fn active_is_docx(&self) -> bool {
        self.active()
            .is_some_and(|session| matches!(session.editor, EditorSession::WordCraft(_)))
    }
    pub fn active_csv_mut(&mut self) -> Option<&mut CsvEditorUi> {
        match &mut self.sessions.get_mut(self.active?)?.editor {
            EditorSession::Csv(editor) => Some(editor),
            _ => None,
        }
    }
    #[cfg(feature = "embedded-wordcraft")]
    pub fn active_docx_mut(&mut self) -> Option<&mut WordCraftEmbed> {
        match &mut self.sessions.get_mut(self.active?)?.editor {
            EditorSession::WordCraft(editor) => Some(editor),
            _ => None,
        }
    }

    /// A failed load leaves the previously active session and every editor intact.
    pub fn open(&mut self, path: &Path) -> Result<(), String> {
        let path = path
            .canonicalize()
            .map_err(|e| format!("파일 경로 확인 실패: {e}"))?;
        if !path.is_file() {
            return Err("일반 파일만 열 수 있습니다.".into());
        }
        let format = FormatKind::from_path(&path)
            .ok_or_else(|| "아직 지원하지 않는 파일 확장자입니다.".to_string())?;
        if let Some(index) = self
            .sessions
            .iter()
            .position(|session| session.path == path)
        {
            self.active = Some(index);
            return Ok(());
        }
        let editor = match format {
            FormatKind::Csv => EditorSession::Csv(CsvEditorUi::open(&path)?),
            #[cfg(feature = "embedded-wordcraft")]
            FormatKind::Docx => EditorSession::WordCraft(WordCraftEmbed::open(&path)?),
            _ => EditorSession::ExternalOnly,
        };
        self.sessions.push(DocumentSession {
            path,
            format,
            editor,
        });
        self.active = Some(self.sessions.len() - 1);
        Ok(())
    }

    /// CSV Save As returns a new path after the file has been written.
    pub fn rename_active_csv(&mut self, path: &Path) -> Result<(), String> {
        let path = path.canonicalize().map_err(|e| e.to_string())?;
        let index = self.active.ok_or("활성 문서가 없습니다.")?;
        if !matches!(self.sessions[index].editor, EditorSession::Csv(_)) {
            return Err("CSV 세션이 아닙니다.".into());
        }
        if self
            .sessions
            .iter()
            .enumerate()
            .any(|(i, s)| i != index && s.path == path)
        {
            return Err("다른 탭에서 이미 열린 파일입니다.".into());
        }
        self.sessions[index].path = path;
        Ok(())
    }

    /// Caller must obtain explicit user consent before dropping unsaved changes.
    pub fn close_active(&mut self) {
        if let Some(index) = self.active {
            self.sessions.remove(index);
            self.active = if self.sessions.is_empty() {
                None
            } else {
                Some(index.min(self.sessions.len() - 1))
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn two_csv_editors_survive_tab_switches() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.csv");
        let b = dir.path().join("b.csv");
        fs::write(&a, "x\n1\n").unwrap();
        fs::write(&b, "y\n2\n").unwrap();
        let mut sessions = DocumentSessions::default();
        sessions.open(&a).unwrap();
        assert!(sessions.active_csv_mut().is_some());
        sessions.open(&b).unwrap();
        assert_eq!(sessions.sessions().len(), 2);
        sessions.open(&a).unwrap();
        assert_eq!(sessions.active_index(), Some(0));
        assert!(sessions.active_csv_mut().is_some());
    }

    #[test]
    fn failed_open_does_not_discard_active_editor() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.csv");
        fs::write(&a, "x\n1\n").unwrap();
        let mut sessions = DocumentSessions::default();
        sessions.open(&a).unwrap();
        assert!(sessions.open(&dir.path().join("missing.xlsx")).is_err());
        assert_eq!(sessions.sessions().len(), 1);
        assert!(sessions.active_csv_mut().is_some());
    }

    #[test]
    fn external_document_is_not_misrepresented_as_editor() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("slides.pptx");
        fs::write(&file, "placeholder").unwrap();
        let mut sessions = DocumentSessions::default();
        sessions.open(&file).unwrap();
        assert_eq!(
            sessions.active().unwrap().adapter(),
            AdapterKind::ExternalOnly
        );
        sessions.close_active();
        assert!(sessions.active().is_none());
    }
}
