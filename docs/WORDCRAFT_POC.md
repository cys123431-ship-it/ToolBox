# WordCraft 내부 UI 실험

`cargo run --features embedded-wordcraft`로 실행하면 DOCX를 임시 복사본으로 열어 WordCraft의 egui UI를 ToolBox 창 안에서 호출한다. **현재 원본 저장 기능은 제공하지 않는다.** 사용자가 '새 DOCX 파일로 내보내기'를 선택하면 새로운 DOCX 파일을 만들 수 있으며 기존 파일은 덮어쓰지 않는다. 임시 사본은 편집기를 닫으면 삭제되므로 내보내지 않은 편집 내용은 유실된다.

WordCraft: https://github.com/storytold/wordcraft ; 고정 리비전 `e6c5a701ddff5be16e266028168c70658aae39d9`. MIT OR Apache-2.0 및 전이 의존성 고지를 릴리스 전에 확인한다.

**미검증:** Windows GUI 빌드/실행, 한글 IME, 리본·포커스, 실제 DOCX 열기→수정→저장→재열기, 서식·표·각주·이미지·수식 보존, Microsoft Word 호환성. 원본 반영 승인·백업·복구는 후속 개발 대상.

## DOCX ZIP 구조·무결성 검증

`embedded-wordcraft` 옵션에서 DOCX 원본을 열기 전, WordCraft가 임시 사본을 저장한 직후, 새 파일로 내보내기 전에 ZIP 구조 검사를 수행한다. ZIP 항목의 CRC를 끝까지 읽어 검사하고 필수 OOXML 구성 요소 3개(`[Content_Types].xml`, `_rels/.rels`, `word/document.xml`)가 있는지 확인한다. 경로 탐색(`../`), 중복 항목, 항목 수/압축 해제 크기 초과는 거부한다. 검사 실패 시 새 파일을 생성하지 않는다. 유효한 OOXML 전체 규격 검사·렌더링·Microsoft Word 왕복 검증은 **아직 수행하지 않는다**. 테스트는 최소 OOXML ZIP 샘플과 손상·누락·경로 탐색·중복 항목을 검증한다.

## 2026-10-20 ZIP 테스트 보정

WordCraft 기능 테스트가 `duplicate_parts_are_rejected`에서 실패했다. 원인은 검증기가 아니라 테스트 준비 단계에서 ZIP 2.x의 `ZipWriter`가 중복 파일명 작성을 거부하기 때문이다. 이 테스트는 라이브러리의 중복 파일명 작성 거부를 검증하도록 수정했다. **검증기의 중복 파일명 거부 경로는 아직 별도의 외부 제작 ZIP 테스트 데이터가 필요하다.** 또한 실제 ZIP 내용의 한 바이트를 변조한 CRC 오류 검출 회귀 테스트를 추가했다. 이들은 OOXML 의미·서식 호환성 테스트가 아니다.
