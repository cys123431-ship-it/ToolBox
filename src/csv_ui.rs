//! Native CSV/TSV grid and cell editor. No external editor process is required.
use crate::csv_recovery::{self, RecoverySnapshot};
use crate::csv_workspace::CsvWorkspace;
use eframe::egui;
use std::path::{Path, PathBuf};

pub struct CsvEditorUi {
    pub workspace: CsvWorkspace,
    selected: Option<(usize, usize)>,
    draft: String,
    notice: String,
    pending_recovery: Option<RecoverySnapshot>,
    recovery_conflict: bool,
}

impl CsvEditorUi {
    pub fn open(path: &Path) -> Result<Self, String> {
        let workspace = CsvWorkspace::open(path)?;
        let selected = if workspace.rows.is_empty() {
            None
        } else {
            Some((0, 0))
        };
        let draft = workspace.cell(0, 0).to_owned();
        Ok(Self {
            workspace,
            selected,
            draft,
            notice: String::new(),
        })
    }

    pub fn is_dirty(&self) -> bool {
        self.workspace.is_dirty() || self.has_pending_draft()
    }

    fn has_pending_draft(&self) -> bool {
        self.selected
            .is_some_and(|(r, c)| self.draft != self.workspace.cell(r, c))
    }

    fn choose(&mut self, row: usize, column: usize) {
        if self.has_pending_draft() {
            self.notice = "현재 셀의 내용을 적용하거나 취소한 뒤 이동하세요.".into();
            return;
        }
        self.selected = Some((row, column));
        self.draft = self.workspace.cell(row, column).to_owned();
    }

    fn refresh_selection(&mut self) {
        self.selected = self
            .selected
            .filter(|(r, c)| *r < self.workspace.rows.len() && *c < self.workspace.width());
        if self.selected.is_none() && !self.workspace.rows.is_empty() {
            self.selected = Some((0, 0));
        }
        self.draft = self
            .selected
            .map(|(r, c)| self.workspace.cell(r, c).to_owned())
            .unwrap_or_default();
    }

    fn apply_draft(&mut self) -> Result<(), String> {
        if let Some((row, col)) = self.selected {
            self.workspace.set_cell(row, col, self.draft.clone())?;
        }
        Ok(())
    }

    /// Returns the new document path when Save As succeeds.
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<PathBuf> {
        let mut saved_as = None;
        ui.heading("CSV / TSV 내부 편집기");
        ui.label(format!("파일: {}", self.workspace.path.display()));
        ui.label(format!(
            "{}행 × {}열 · {} · {}",
            self.workspace.rows.len(),
            self.workspace.width(),
            if self.workspace.delimiter == b'\t' {
                "TSV"
            } else {
                "CSV"
            },
            if self.is_dirty() {
                "저장하지 않은 변경사항"
            } else {
                "저장됨"
            }
        ));
        ui.small("셀 선택 → 아래 편집창에서 수정 → '셀 적용'. CSV 저장 시 원본 줄바꿈/따옴표 표기는 정규화될 수 있습니다.");

        ui.horizontal_wrapped(|ui| {
            if ui.button("셀 적용").clicked() {
                self.notice = match self.apply_draft() {
                    Ok(()) => "셀 내용을 적용했습니다.".into(),
                    Err(e) => e,
                };
            }
            if ui.button("셀 입력 취소").clicked() {
                self.refresh_selection();
                self.notice = "미적용 셀 입력을 취소했습니다.".into();
            }
            if ui
                .add_enabled(
                    self.workspace.can_undo() && !self.has_pending_draft(),
                    egui::Button::new("실행 취소"),
                )
                .clicked()
            {
                self.workspace.undo();
                self.refresh_selection();
            }
            if ui
                .add_enabled(
                    self.workspace.can_redo() && !self.has_pending_draft(),
                    egui::Button::new("다시 실행"),
                )
                .clicked()
            {
                self.workspace.redo();
                self.refresh_selection();
            }
            if ui
                .add_enabled(!self.has_pending_draft(), egui::Button::new("행 추가"))
                .clicked()
            {
                self.notice = match self.workspace.add_row() {
                    Ok(()) => "행을 추가했습니다.".into(),
                    Err(e) => e,
                };
                self.refresh_selection();
            }
            if ui
                .add_enabled(!self.has_pending_draft(), egui::Button::new("열 추가"))
                .clicked()
            {
                self.notice = match self.workspace.add_column() {
                    Ok(()) => "열을 추가했습니다.".into(),
                    Err(e) => e,
                };
                self.refresh_selection();
            }
            if ui
                .add_enabled(
                    self.selected.is_some() && !self.has_pending_draft(),
                    egui::Button::new("선택 행 삭제"),
                )
                .clicked()
            {
                if let Some((row, _)) = self.selected {
                    self.notice = match self.workspace.delete_row(row) {
                        Ok(()) => "행을 삭제했습니다. 실행 취소할 수 있습니다.".into(),
                        Err(e) => e,
                    };
                    self.refresh_selection();
                }
            }
        });

        ui.horizontal(|ui| {
            if ui.button("현재 파일 저장 (백업 생성)").clicked() {
                self.notice = match self
                    .apply_draft()
                    .and_then(|_| self.workspace.save_current())
                {
                    Ok(Some(backup)) => format!("저장 완료. 백업: {}", backup.display()),
                    Ok(None) => "변경 사항이 없습니다.".into(),
                    Err(e) => format!("저장 실패: {e}"),
                };
            }
            if ui.button("다른 이름으로 저장…").clicked() {
                let suggested = self
                    .workspace
                    .path
                    .file_name()
                    .map(|s| format!("copy-{}", s.to_string_lossy()))
                    .unwrap_or_else(|| "copy.csv".into());
                if let Some(target) = rfd::FileDialog::new()
                    .set_title("CSV/TSV 새 파일 저장")
                    .set_file_name(&suggested)
                    .save_file()
                {
                    self.notice = match self
                        .apply_draft()
                        .and_then(|_| self.workspace.save_as(&target))
                    {
                        Ok(()) => {
                            saved_as = Some(target);
                            "새 파일을 저장했습니다. 기존 파일은 그대로 유지됩니다.".into()
                        }
                        Err(e) => format!("다른 이름으로 저장 실패: {e}"),
                    };
                }
            }
        });

        ui.separator();
        let row_count = self.workspace.rows.len();
        let columns = self.workspace.width();
        ui.label("데이터 표 (행·열을 선택하면 아래에서 내용을 편집할 수 있습니다)");
        let mut chosen = None;
        egui::ScrollArea::both()
            .max_height(290.0)
            .show_rows(ui, 27.0, row_count, |ui, range| {
                for row in range {
                    ui.horizontal(|ui| {
                        ui.add_sized([42.0, 24.0], egui::Label::new((row + 1).to_string()));
                        for col in 0..columns {
                            let value = self.workspace.cell(row, col);
                            let preview: String = value
                                .chars()
                                .take(25)
                                .collect::<String>()
                                .replace('\n', "↵")
                                .replace('\r', "");
                            let label = if preview.is_empty() {
                                "·".into()
                            } else {
                                preview
                            };
                            if ui
                                .add_sized(
                                    [135.0, 24.0],
                                    egui::Button::new(label)
                                        .selected(self.selected == Some((row, col))),
                                )
                                .clicked()
                            {
                                chosen = Some((row, col));
                            }
                        }
                    });
                }
            });
        if let Some((row, col)) = chosen {
            self.choose(row, col);
        }

        ui.separator();
        if let Some((row, col)) = self.selected {
            ui.label(format!("선택 셀: {}행 {}열", row + 1, col + 1));
            ui.add_sized(
                [ui.available_width(), 70.0],
                egui::TextEdit::multiline(&mut self.draft).hint_text("선택한 셀의 값"),
            );
            if self.has_pending_draft() {
                ui.colored_label(
                    egui::Color32::from_rgb(175, 100, 20),
                    "셀 입력이 아직 적용되지 않았습니다.",
                );
            }
        } else {
            ui.weak("빈 파일입니다. '행 추가'로 시작하세요.");
        }
        if !self.notice.is_empty() {
            ui.label(&self.notice);
        }
        saved_as
    }
}
