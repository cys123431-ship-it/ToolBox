use crate::csv_ui::CsvEditorUi;
use crate::engines::{self, Engine};
use crate::preferences::Preferences;
use eframe::egui;
use std::path::PathBuf;

pub struct ToolBoxApp {
    preferences: Preferences,
    csv_editor: Option<CsvEditorUi>,
    active: Engine,
    selected: Option<PathBuf>,
    notice: String,
    show_settings: bool,
}

impl ToolBoxApp {
    pub fn new() -> Self {
        Self {
            preferences: Preferences::load(),
            csv_editor: None,
            active: Engine::Document,
            selected: None,
            notice: "현재 버전은 편집기 연결 실험용입니다. 통합 편집기는 개발 중입니다.".into(),
            show_settings: false,
        }
    }

    fn select(&mut self, path: PathBuf) {
        if let Some(editor) = self.csv_editor.as_ref() {
            if editor.workspace.path == path {
                self.active = Engine::Spreadsheet;
                return;
            }
            if editor.is_dirty() {
                self.notice =
                    "CSV/TSV의 변경 사항을 저장하거나 편집기를 닫은 뒤 다른 파일을 선택하세요."
                        .into();
                return;
            }
        }
        match Engine::from_path(&path) {
            Some(engine) => {
                let is_csv = path.extension().and_then(|s| s.to_str()).is_some_and(|s| {
                    s.eq_ignore_ascii_case("csv") || s.eq_ignore_ascii_case("tsv")
                });
                if is_csv {
                    match CsvEditorUi::open(&path) {
                        Ok(editor) => self.csv_editor = Some(editor),
                        Err(error) => {
                            self.notice = format!("CSV/TSV 열기 실패: {error}");
                            return;
                        }
                    }
                } else {
                    self.csv_editor = None;
                }
                self.active = engine;
                self.selected = Some(path.clone());
                self.preferences.record_recent(&path);
                if let Err(error) = self.preferences.save() {
                    self.notice = format!("최근 파일 목록 저장 실패: {error}");
                } else {
                    self.notice = format!("선택됨: {}", path.display());
                }
            }
            None => self.notice = "아직 지원하지 않는 파일 확장자입니다.".into(),
        }
    }

    fn launch(&mut self, path: PathBuf) {
        let Some(engine) = Engine::from_path(&path) else {
            self.notice = "알 수 없는 파일 형식입니다.".into();
            return;
        };
        let binary = engines::resolve_binary(engine, self.preferences.binary(engine.key()));
        let Some(binary) = binary else {
            self.notice = if engine == Engine::Hangul {
                "HWP/HWPX 데스크톱 편집 브리지는 아직 구현되지 않았습니다.".into()
            } else {
                format!(
                    "{} 실행 파일이 없습니다. 설정에서 경로를 지정하세요.",
                    engine.key()
                )
            };
            return;
        };
        match engines::launch_document(engine, &binary, &path) {
            Ok(()) => {
                self.notice = format!("{}에 파일 열기를 요청했습니다.", engine.key());
                self.preferences.record_recent(&path);
                if let Err(e) = self.preferences.save() {
                    self.notice.push_str(&format!(" (기록 저장 실패: {e})"));
                }
            }
            Err(err) => self.notice = err,
        }
    }

    fn module_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.active.title());
        ui.label(self.active.description());
        ui.add_space(12.0);
        let binary =
            engines::resolve_binary(self.active, self.preferences.binary(self.active.key()));
        if self.active == Engine::Hangul {
            ui.colored_label(
                egui::Color32::from_rgb(195, 120, 35),
                "준비 중 · rHWP는 별도 편집기 브리지 구현이 필요합니다.",
            );
        } else {
            ui.label(match &binary {
                Some(path) => format!("편집기 탐지됨: {}", path.display()),
                None => format!("편집기 미설치 또는 경로 미설정: {}", self.active.key()),
            });
        }

        ui.add_space(10.0);
        if ui.button("파일 선택…").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("ToolBox - 파일 선택")
                .pick_file()
            {
                self.select(path);
            }
        }
        if self.active == Engine::Spreadsheet && self.csv_editor.is_some() {
            let saved_as = self.csv_editor.as_mut().and_then(|editor| editor.show(ui));
            if let Some(path) = saved_as {
                self.selected = Some(path.clone());
                self.preferences.record_recent(&path);
                if let Err(error) = self.preferences.save() {
                    self.notice = format!("최근 파일 목록 저장 실패: {error}");
                }
            }
            let dirty = self.csv_editor.as_ref().is_some_and(CsvEditorUi::is_dirty);
            let pending = self
                .csv_editor
                .as_ref()
                .is_some_and(CsvEditorUi::has_pending_recovery);
            ui.add_space(10.0);
            if ui
                .button(if dirty {
                    "저장하지 않은 변경을 버리고 CSV/TSV 닫기"
                } else {
                    if pending {
                        "CSV/TSV 닫기 (복구 데이터 유지)"
                    } else {
                        "CSV/TSV 닫기"
                    }
                })
                .clicked()
            {
                let result = self.csv_editor.as_mut().map(CsvEditorUi::discard_changes);
                if let Some(Err(e)) = result {
                    self.notice = format!("복구 데이터 삭제 실패: {e}");
                } else {
                    self.csv_editor = None;
                    self.selected = None;
                    self.notice = "CSV/TSV 편집기를 닫았습니다.".into();
                }
            }
            return;
        }
        if let Some(path) = self.selected.clone() {
            ui.label(format!("선택한 파일: {}", path.display()));
            if Engine::from_path(&path) == Some(self.active)
                && ui
                    .add_enabled(
                        self.active.can_launch_document() && binary.is_some(),
                        egui::Button::new("선택 파일을 편집기로 열기"),
                    )
                    .clicked()
            {
                self.launch(path);
            }
        } else {
            ui.weak("문서 파일을 선택하거나 왼쪽 최근 파일 목록에서 선택하세요.");
        }

        ui.add_space(16.0);
        ui.separator();
        ui.heading("진행 상태");
        ui.label("현재: 독립 편집기 실행 · 최근 파일 · 사용자 지정 실행 경로");
        ui.label("다음: 동일 창 내부 편집, 파일 왕복 테스트, AI 작업 승인");
        if self.show_settings {
            ui.separator();
            ui.heading("편집기 실행 파일 설정");
            if !self.active.can_launch_document() {
                ui.label("이 모듈은 아직 경로 설정 대상이 아닙니다.");
            } else {
                let value = self
                    .preferences
                    .executables
                    .entry(self.active.key().to_string())
                    .or_default();
                ui.label("실행 파일 전체 경로 (미지정 시 PATH에서 탐색)");
                ui.text_edit_singleline(value);
                ui.horizontal(|ui| {
                    if ui.button("실행 파일 찾기…").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.preferences.executables.insert(
                                self.active.key().to_string(),
                                path.to_string_lossy().into_owned(),
                            );
                        }
                    }
                    if ui.button("설정 저장").clicked() {
                        self.notice = match self.preferences.save() {
                            Ok(()) => "설정을 저장했습니다.".into(),
                            Err(e) => format!("설정 저장 실패: {e}"),
                        };
                    }
                });
            }
        }
    }
}

impl eframe::App for ToolBoxApp {
    fn ui(&mut self, root_ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(path) = root_ui.input(|input| {
            input.raw.dropped_files.iter().find_map(|file| {
                let path = file.path();
                path.is_absolute().then(|| path.to_path_buf())
            })
        }) {
            self.select(path);
        }
        egui::Panel::top("top").show(root_ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("ToolBox");
                ui.separator();
                ui.label("Desktop Alpha · 로컬 우선");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(if self.show_settings {
                            "설정 닫기"
                        } else {
                            "설정"
                        })
                        .clicked()
                    {
                        self.show_settings = !self.show_settings;
                    }
                });
            });
        });
        egui::Panel::left("modules")
            .resizable(false)
            .default_size(240.0)
            .show(root_ui, |ui| {
                ui.heading("작업 공간");
                ui.separator();
                for engine in Engine::ALL {
                    if ui
                        .selectable_label(self.active == engine, engine.title())
                        .clicked()
                    {
                        self.active = engine;
                    }
                }
                ui.add_space(22.0);
                ui.heading("최근 파일");
                let recent = self.preferences.recent.clone();
                egui::ScrollArea::vertical()
                    .max_height(330.0)
                    .show(ui, |ui| {
                        for path in recent {
                            let name = path
                                .file_name()
                                .map(|s| s.to_string_lossy().into_owned())
                                .unwrap_or_else(|| path.display().to_string());
                            if ui
                                .button(name)
                                .on_hover_text(path.display().to_string())
                                .clicked()
                            {
                                self.select(path);
                            }
                        }
                    });
            });
        egui::Panel::bottom("status").show(root_ui, |ui| {
            ui.label(&self.notice);
        });
        egui::CentralPanel::default().show(root_ui, |ui| self.module_panel(ui));
    }
}
