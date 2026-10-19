//! Explicitly reviewed CSV/TSV cell-edit plans. No network, shell, or file writes.
//! This is a narrow foundation for future AI/MCP tools, not an AI integration.

use crate::csv_workspace::{CellChange, CsvWorkspace, MAX_BYTES};
use serde::Deserialize;
use std::collections::HashSet;

pub const MAX_PLAN_BYTES: usize = 256 * 1024;
pub const MAX_PLAN_EDITS: usize = 200;
const PLAN_VERSION: u8 = 1;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CsvEditPlan {
    pub version: u8,
    pub source_sha256: String,
    pub changes: Vec<CellChange>,
}

#[derive(Debug, Clone)]
pub struct CsvPlanPreview {
    pub changes: Vec<CellChange>,
    source_sha256: String,
    rows_sha256: String,
}

fn rows_digest(workspace: &CsvWorkspace) -> Result<String, String> {
    let bytes = serde_json::to_vec(&workspace.rows).map_err(|e| e.to_string())?;
    Ok(crate::csv_recovery::digest(&bytes))
}

impl CsvEditPlan {
    pub fn parse(json: &str) -> Result<Self, String> {
        if json.len() > MAX_PLAN_BYTES {
            return Err("변경 계획은 최대 256KB까지 허용됩니다.".into());
        }
        let plan: Self =
            serde_json::from_str(json).map_err(|e| format!("변경 계획 JSON 오류: {e}"))?;
        if plan.version != PLAN_VERSION {
            return Err("지원하지 않는 변경 계획 버전입니다.".into());
        }
        if plan.changes.is_empty() || plan.changes.len() > MAX_PLAN_EDITS {
            return Err(format!("변경 계획은 1~{MAX_PLAN_EDITS}개 셀만 허용합니다."));
        }
        if plan.source_sha256.len() != 64
            || !plan.source_sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("원본 SHA-256 확인값이 올바르지 않습니다.".into());
        }
        Ok(plan)
    }

    pub fn preview(self, workspace: &CsvWorkspace) -> Result<CsvPlanPreview, String> {
        if self.source_sha256 != workspace.source_sha256() {
            return Err("계획의 원본 SHA-256이 현재 파일과 다릅니다.".into());
        }
        validate_changes(workspace, &self.changes)?;
        Ok(CsvPlanPreview {
            changes: self.changes,
            source_sha256: self.source_sha256,
            rows_sha256: rows_digest(workspace)?,
        })
    }
}

fn validate_changes(workspace: &CsvWorkspace, changes: &[CellChange]) -> Result<(), String> {
    let mut seen = HashSet::new();
    let mut proposed = workspace.rows.clone();
    if changes.is_empty() || changes.len() > MAX_PLAN_EDITS {
        return Err("변경 계획의 셀 개수가 허용 범위를 벗어납니다.".into());
    }
    for change in changes {
        if !seen.insert((change.row, change.col)) {
            return Err("같은 셀을 두 번 변경하는 계획은 허용되지 않습니다.".into());
        }
        if change.row >= workspace.rows.len() || change.col >= crate::csv_workspace::MAX_COLUMNS {
            return Err("변경 계획에 유효하지 않은 셀 위치가 있습니다.".into());
        }
        if workspace.cell(change.row, change.col) != change.expected {
            return Err(format!(
                "{}행 {}열의 현재 값이 계획의 예상 값과 다릅니다.",
                change.row + 1,
                change.col + 1
            ));
        }
        if change.expected == change.replacement {
            return Err("변경 전후 값이 동일한 셀이 포함되어 있습니다.".into());
        }
        let row = &mut proposed[change.row];
        if change.col >= row.len() {
            row.resize(change.col + 1, String::new());
        }
        row[change.col] = change.replacement.clone();
    }
    let total = proposed
        .iter()
        .flat_map(|row| row.iter())
        .try_fold(0usize, |sum, cell| sum.checked_add(cell.len()))
        .ok_or("변경 결과가 너무 큽니다.")?;
    if total > MAX_BYTES {
        return Err("변경 결과가 2MB를 초과합니다.".into());
    }
    Ok(())
}

impl CsvPlanPreview {
    /// Must be called only after the user presses the explicit approval button.
    /// Re-checks the whole workspace fingerprint, not just edited cells.
    pub fn apply(&self, workspace: &mut CsvWorkspace) -> Result<usize, String> {
        if self.source_sha256 != workspace.source_sha256()
            || self.rows_sha256 != rows_digest(workspace)?
        {
            return Err("미리보기 이후 표가 변경됐습니다. 다시 검토하세요.".into());
        }
        validate_changes(workspace, &self.changes)?;
        workspace.apply_batch_cells(&self.changes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn fixture() -> (tempfile::TempDir, CsvWorkspace) {
        let dir = tempdir().unwrap();
        let file = dir.path().join("sample.csv");
        fs::write(&file, "이름,점수\n철수,80\n영희,90\n").unwrap();
        let workspace = CsvWorkspace::open(&file).unwrap();
        (dir, workspace)
    }

    fn plan_json(workspace: &CsvWorkspace, changes: &str) -> String {
        format!(
            r#"{{"version":1,"source_sha256":"{}","changes":{changes}}}"#,
            workspace.source_sha256()
        )
    }

    #[test]
    fn preview_does_not_modify_disk_and_approval_is_one_undo_step() {
        let (_dir, mut workspace) = fixture();
        let original = fs::read(&workspace.path).unwrap();
        let json = plan_json(
            &workspace,
            r#"[{"row":1,"col":1,"expected":"80","replacement":"85"},{"row":2,"col":1,"expected":"90","replacement":"95"}]"#,
        );
        let preview = CsvEditPlan::parse(&json)
            .unwrap()
            .preview(&workspace)
            .unwrap();
        assert_eq!(workspace.cell(1, 1), "80");
        assert_eq!(fs::read(&workspace.path).unwrap(), original);
        assert_eq!(preview.apply(&mut workspace).unwrap(), 2);
        assert_eq!(workspace.cell(1, 1), "85");
        assert_eq!(workspace.cell(2, 1), "95");
        assert!(workspace.undo());
        assert_eq!(workspace.cell(1, 1), "80");
        assert_eq!(workspace.cell(2, 1), "90");
        assert_eq!(fs::read(&workspace.path).unwrap(), original);
    }

    #[test]
    fn rejects_wrong_hash_duplicate_and_unknown_fields() {
        let (_dir, workspace) = fixture();
        let wrong_hash = r#"{"version":1,"source_sha256":"0000000000000000000000000000000000000000000000000000000000000000","changes":[{"row":1,"col":1,"expected":"80","replacement":"85"}]}"#;
        assert!(CsvEditPlan::parse(wrong_hash)
            .unwrap()
            .preview(&workspace)
            .is_err());
        let duplicate = plan_json(
            &workspace,
            r#"[{"row":1,"col":1,"expected":"80","replacement":"85"},{"row":1,"col":1,"expected":"80","replacement":"86"}]"#,
        );
        assert!(CsvEditPlan::parse(&duplicate)
            .unwrap()
            .preview(&workspace)
            .is_err());
        let unknown = plan_json(
            &workspace,
            r#"[{"row":1,"col":1,"expected":"80","replacement":"85","command":"shell"}]"#,
        );
        assert!(CsvEditPlan::parse(&unknown).is_err());
    }

    #[test]
    fn rejects_stale_preview_and_preserves_existing_edits() {
        let (_dir, mut workspace) = fixture();
        let json = plan_json(
            &workspace,
            r#"[{"row":1,"col":1,"expected":"80","replacement":"85"}]"#,
        );
        let preview = CsvEditPlan::parse(&json)
            .unwrap()
            .preview(&workspace)
            .unwrap();
        workspace.set_cell(2, 1, "91".into()).unwrap();
        assert!(preview.apply(&mut workspace).is_err());
        assert_eq!(workspace.cell(1, 1), "80");
        assert_eq!(workspace.cell(2, 1), "91");
    }

    #[test]
    fn invalid_expected_cell_or_bounds_are_rejected_atomically() {
        let (_dir, workspace) = fixture();
        let json = plan_json(
            &workspace,
            r#"[{"row":1,"col":1,"expected":"80","replacement":"85"},{"row":999,"col":0,"expected":"","replacement":"x"}]"#,
        );
        let preview = CsvEditPlan::parse(&json).unwrap().preview(&workspace);
        assert!(preview.is_err());
        assert_eq!(workspace.cell(1, 1), "80");
        assert!(!workspace.can_undo());
        let stale = plan_json(
            &workspace,
            r#"[{"row":1,"col":1,"expected":"70","replacement":"85"}]"#,
        );
        assert!(CsvEditPlan::parse(&stale)
            .unwrap()
            .preview(&workspace)
            .is_err());
    }

    #[test]
    fn malformed_and_oversized_plans_are_rejected() {
        assert!(CsvEditPlan::parse("{}").is_err());
        assert!(CsvEditPlan::parse(&"x".repeat(MAX_PLAN_BYTES + 1)).is_err());
    }
}
