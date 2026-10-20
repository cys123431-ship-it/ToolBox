# WordCraft in-process API 검토 (2026-10-20)

WordCraft의 `wordcraft-ui-egui`는 `WordApp::new(Session, Services)`와 `WordApp::ui(&mut egui::Ui)`를 공개한다. `wordcraft-engine`의 `Session::new(Document)` 및 `WordApp::run("file.open", {"path": ...})`를 사용해 ToolBox의 동일 egui 창에서 DOCX를 여는 기술적 연결을 실험할 수 있다.

검토 대상: https://github.com/storytold/wordcraft (commit e6c5a701ddff5be16e266028168c70658aae39d9). Rust 1.95 이상, egui 0.36 필요. 외부 WordCraft 앱 실행은 단일 창 편집이 아니다.

실제 통합에는 독립된 WordCraft UI의 패널/포커스 조정, 원본 파일 보호와 백업, 사용자 승인, 편집 중 파일 변경 충돌 검사, DOCX 왕복 호환성 검증이 필요하다. 구현과 Windows 빌드가 완료되기 전까지 내장 DOCX 편집은 미구현으로 표시한다.
