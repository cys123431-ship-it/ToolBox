# WordCraft 내부 UI 실험

`cargo run --features embedded-wordcraft`로 실행하면 DOCX를 임시 복사본으로 열어 WordCraft의 egui UI를 ToolBox 창 안에서 호출한다. **현재 원본 저장 기능은 제공하지 않는다.** 사용자가 '새 DOCX 파일로 내보내기'를 선택하면 새로운 DOCX 파일을 만들 수 있으며 기존 파일은 덮어쓰지 않는다. 임시 사본은 편집기를 닫으면 삭제되므로 내보내지 않은 편집 내용은 유실된다.

WordCraft: https://github.com/storytold/wordcraft ; 고정 리비전 `e6c5a701ddff5be16e266028168c70658aae39d9`. MIT OR Apache-2.0 및 전이 의존성 고지를 릴리스 전에 확인한다.

**미검증:** Windows GUI 빌드/실행, 한글 IME, 리본·포커스, 실제 DOCX 열기→수정→저장→재열기, 서식·표·각주·이미지·수식 보존, Microsoft Word 호환성. 원본 반영 승인·백업·복구는 후속 개발 대상.
