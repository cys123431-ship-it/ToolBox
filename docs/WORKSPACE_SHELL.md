# Windows 공통 작업공간 1차 골격

2026-10-20 구현. `src/workspace.rs`는 DOCX/HWP/HWPX/XLSX/CSV/PPTX/PDF 7종을 구분하고, 현재 사용 가능한 편집 어댑터의 능력을 표시한다. 기존 egui 단일 창 상단에 파일 열기·형식/엔진 상태·가로 작업 탭을 추가한다. 기존 CSV/TSV 편집기와 선택적 WordCraft DOCX 실험을 재사용한다.

**중요한 한계:** 작업 탭은 파일 경로를 기억하는 UI 목록이다. 독립된 동시 편집 세션을 제공하지 않는다. 탭 선택은 기존 `select`를 다시 호출하여 CSV 미저장 변경 및 WordCraft 임시 편집 폐기 보호를 유지한다. CSV만 ToolBox 내부 원본 저장을 제공한다. WordCraft DOCX는 `embedded-wordcraft` 기능을 켠 경우 임시 사본 편집·새 파일 내보내기만 지원한다. XLSX/PPTX/PDF는 외부 실행 연결 단계이고, HWP/HWPX는 미구현이다. 미지원 형식의 공통 저장 버튼은 제공하지 않는다.

다음 큰 단계: 문서별 독립 `DocumentSession` 및 실질적인 편집기 `open/edit/save/export/close` 계약, GridCraft/DeckCraft/rHWP/PdfCraft 내장 연결, Windows GUI/실파일 왕복 검증. 코드가 컴파일되더라도 실제 Windows GUI 사용성은 별도 검증이 필요하다.
