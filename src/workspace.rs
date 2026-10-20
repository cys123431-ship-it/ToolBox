//! Shared file routing and lightweight workspace tabs.
//! Tabs record selected paths; they do not represent simultaneous live editor sessions.
use crate::engines::Engine;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatKind {
    Docx,
    Hwp,
    Hwpx,
    Xlsx,
    Csv,
    Pptx,
    Pdf,
    OtherDocument,
    OtherSheet,
    Hml,
}

impl FormatKind {
    pub const CORE: [Self; 7] = [
        Self::Docx,
        Self::Hwp,
        Self::Hwpx,
        Self::Xlsx,
        Self::Csv,
        Self::Pptx,
        Self::Pdf,
    ];

    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        Some(match ext.as_str() {
            "docx" => Self::Docx,
            "hwp" => Self::Hwp,
            "hwpx" => Self::Hwpx,
            "xlsx" => Self::Xlsx,
            "csv" | "tsv" => Self::Csv,
            "pptx" => Self::Pptx,
            "pdf" => Self::Pdf,
            "doc" | "odt" | "rtf" | "md" | "txt" => Self::OtherDocument,
            "xls" => Self::OtherSheet,
            "hml" => Self::Hml,
            _ => return None,
        })
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Docx => "DOCX",
            Self::Hwp => "HWP",
            Self::Hwpx => "HWPX",
            Self::Xlsx => "XLSX",
            Self::Csv => "CSV/TSV",
            Self::Pptx => "PPTX",
            Self::Pdf => "PDF",
            Self::OtherDocument => "기타 문서",
            Self::OtherSheet => "기타 시트",
            Self::Hml => "HML",
        }
    }

    pub const fn engine(self) -> Engine {
        match self {
            Self::Docx | Self::OtherDocument => Engine::Document,
            Self::Xlsx | Self::Csv | Self::OtherSheet => Engine::Spreadsheet,
            Self::Pptx => Engine::Slides,
            Self::Hwp | Self::Hwpx | Self::Hml => Engine::Hangul,
            Self::Pdf => Engine::Pdf,
        }
    }

    pub const fn adapter(self) -> AdapterKind {
        match self {
            Self::Csv => AdapterKind::NativeCsv,
            Self::Docx if cfg!(feature = "embedded-wordcraft") => {
                AdapterKind::ExperimentalWordCraft
            }
            Self::Hwp | Self::Hwpx | Self::Hml => AdapterKind::Unavailable,
            _ => AdapterKind::ExternalOnly,
        }
    }
}

/// Routing capabilities only; not a claim that a document has loaded successfully.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterKind {
    NativeCsv,
    ExperimentalWordCraft,
    ExternalOnly,
    Unavailable,
}

impl AdapterKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NativeCsv => "창 내부 편집·저장",
            Self::ExperimentalWordCraft => "창 내부 DOCX 실험·새 파일 내보내기",
            Self::ExternalOnly => "외부 편집기 실행만",
            Self::Unavailable => "편집기 연결 미구현",
        }
    }

    pub const fn can_save_original_in_app(self) -> bool {
        matches!(self, Self::NativeCsv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_seven_core_formats() {
        let names = [
            "x.docx", "x.hwp", "x.hwpx", "x.xlsx", "x.csv", "x.pptx", "x.pdf",
        ];
        for (name, kind) in names.iter().zip(FormatKind::CORE) {
            assert_eq!(FormatKind::from_path(Path::new(name)), Some(kind));
            assert_eq!(Engine::from_path(Path::new(name)), Some(kind.engine()));
        }
        assert_eq!(FormatKind::from_path(Path::new("x.TSV")), Some(FormatKind::Csv));
        assert_eq!(FormatKind::from_path(Path::new("x.exe")), None);
    }

    #[test]
    fn only_csv_claims_original_save() {
        assert!(FormatKind::Csv.adapter().can_save_original_in_app());
        for kind in FormatKind::CORE {
            if kind != FormatKind::Csv {
                assert!(!kind.adapter().can_save_original_in_app());
            }
        }
    }
}
