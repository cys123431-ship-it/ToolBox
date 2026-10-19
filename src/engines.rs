//! Registry and safe, argument-based launching for independent open-source editors.
//! No untrusted shell strings are constructed; source-code incorporation is deferred.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Engine {
    Document,
    Spreadsheet,
    Slides,
    Hangul,
    Pdf,
}

impl Engine {
    pub const ALL: [Self; 5] = [
        Self::Document,
        Self::Spreadsheet,
        Self::Slides,
        Self::Hangul,
        Self::Pdf,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Self::Document => "wordcraft",
            Self::Spreadsheet => "gridcraft",
            Self::Slides => "deckcraft",
            Self::Hangul => "rhwp",
            Self::Pdf => "pdfcraft",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Document => "문서 · DOCX",
            Self::Spreadsheet => "스프레드시트 · XLSX",
            Self::Slides => "슬라이드 · PPTX",
            Self::Hangul => "한글 · HWP/HWPX",
            Self::Pdf => "PDF",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Document => "WordCraft를 이용한 Word 문서 열기",
            Self::Spreadsheet => "GridCraft를 이용한 Excel 워크북 열기",
            Self::Slides => "DeckCraft를 이용한 PowerPoint 프레젠테이션 열기",
            Self::Hangul => "rHWP 엔진: 데스크톱 UI 통합 전, 준비 단계",
            Self::Pdf => "PdfCraft를 이용한 PDF 열기",
        }
    }

    pub const fn can_launch_document(self) -> bool {
        !matches!(self, Self::Hangul)
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        Some(match extension.as_str() {
            "docx" | "doc" | "odt" | "rtf" | "md" | "txt" => Self::Document,
            "xlsx" | "xls" | "csv" | "tsv" => Self::Spreadsheet,
            "pptx" => Self::Slides,
            "hwp" | "hwpx" | "hml" => Self::Hangul,
            "pdf" => Self::Pdf,
            _ => return None,
        })
    }
}

/// An explicitly configured path is preferred. Without one, only a real
/// executable file on PATH may be selected (never shell command expansion).
pub fn resolve_binary(engine: Engine, configured: Option<&str>) -> Option<PathBuf> {
    if !engine.can_launch_document() {
        return None;
    }
    if let Some(value) = configured.filter(|v| !v.trim().is_empty()) {
        let path = PathBuf::from(value);
        return path.is_file().then_some(path);
    }
    let filename = if cfg!(windows) {
        format!("{}.exe", engine.key())
    } else {
        engine.key().to_string()
    };
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|dir| dir.join(&filename))
        .find(|p| p.is_file())
}

/// Only launches editors that accept the file path as a positional argument.
/// rHWP requires a separate viewer bridge and is intentionally excluded.
pub fn launch_document(engine: Engine, binary: &Path, document: &Path) -> Result<(), String> {
    if !engine.can_launch_document() {
        return Err("이 형식은 아직 데스크톱 편집기 연결이 지원되지 않습니다.".into());
    }
    if Engine::from_path(document) != Some(engine) {
        return Err("파일 형식과 편집기가 일치하지 않습니다.".into());
    }
    if !document.is_file() {
        return Err("선택한 파일을 찾을 수 없습니다.".into());
    }
    if !binary.is_file() {
        return Err("편집기 실행 파일이 존재하지 않습니다.".into());
    }
    Command::new(binary)
        .arg(document)
        .spawn()
        .map_err(|e| format!("편집기 실행 실패: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_file_extensions_case_insensitively() {
        assert_eq!(
            Engine::from_path(Path::new("draft.DOCX")),
            Some(Engine::Document)
        );
        assert_eq!(
            Engine::from_path(Path::new("grades.xlsx")),
            Some(Engine::Spreadsheet)
        );
        assert_eq!(
            Engine::from_path(Path::new("talk.PPTX")),
            Some(Engine::Slides)
        );
        assert_eq!(
            Engine::from_path(Path::new("form.HWPX")),
            Some(Engine::Hangul)
        );
        assert_eq!(Engine::from_path(Path::new("notes.pdf")), Some(Engine::Pdf));
        assert_eq!(Engine::from_path(Path::new("file.exe")), None);
    }

    #[test]
    fn hangul_is_not_misrepresented_as_a_desktop_launcher() {
        assert!(!Engine::Hangul.can_launch_document());
        assert_eq!(resolve_binary(Engine::Hangul, None), None);
    }

    #[test]
    fn unrecognized_or_mismatched_file_cannot_launch() {
        let bogus_binary = Path::new("nonexistent.exe");
        let bogus_file = Path::new("slides.pptx");
        assert!(launch_document(Engine::Document, bogus_binary, bogus_file).is_err());
    }

    #[test]
    fn missing_custom_binary_is_not_silently_ignored() {
        assert_eq!(
            resolve_binary(Engine::Document, Some("/missing/file/wordcraft.exe")),
            None
        );
    }
}
