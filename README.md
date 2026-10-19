# ToolBox

**Windows-first, local-first desktop workspace for open office editors.**

> **현재 상태: 초기 기술 시제품 (v0.1.0)**  
> 이 저장소는 통합 오피스 완성본이 아닙니다. 현재 구현된 것은 Windows 데스크톱 앱 껍데기, 파일 형식 판별, 설치된 편집기 실행, 최근 파일, 실행 파일 경로 설정입니다. DOCX/XLSX/PPTX 편집은 현재 별도로 설치한 편집기에서 이루어집니다. HWP/HWPX 편집 UI와 ChatGPT 로그인은 아직 구현되지 않았습니다.

## 현재 동작하는 기능

- Rust + egui 기반 데스크톱 창, 한국어 UI(Windows의 맑은 고딕 사용).
- DOCX/문서, XLSX/시트, PPTX/슬라이드, PDF, HWP/HWPX 모듈 화면.
- 파일 선택 및 드래그 앤 드롭, 확장자에 따른 자동 분류.
- WordCraft / GridCraft / DeckCraft / PdfCraft 실행 파일 자동 탐색(PATH) 및 사용자 지정 경로 설정.
- 해당 편집기 프로세스를 쉘 없이 파일 인자로 실행.
- 로컬 최근 파일 목록(최대 15개), 로컬 설정(JSON).
- HWP/HWPX의 연결 미완료 상태를 명확하게 표시.
- Windows 빌드·단위 테스트·실행 파일 업로드용 GitHub Actions.

### 현재 지원/미지원 구분

| 종류 | ToolBox에서 선택 | 외부 앱에서 열기 | ToolBox 내부 편집 |
|---|---|---|---|
| DOCX/DOC/ODT/RTF/MD/TXT | 지원 | WordCraft 설치 시 지원 | 미구현 |
| XLSX/XLS/CSV/TSV | 지원 | GridCraft 설치 시 지원 | 미구현 |
| PPTX | 지원 | DeckCraft 설치 시 지원 | 미구현 |
| PDF | 지원 | PdfCraft 설치 시 지원 | 미구현 |
| HWP/HWPX/HML | 지원 | 미구현 (rHWP 브리지 조사 중) | 미구현 |

이 표는 **외부 앱 실행 연결**만 의미합니다. 오피스 형식의 완전한 왕복 호환성을 보증하지 않습니다.

## 실행 방법 (Windows 11)

1. [Rust](https://rustup.rs/) 안정 버전을 설치합니다 (MSVC 툴체인 권장).
2. 해당 저장소를 clone하고 터미널에서 실행합니다.

    cargo run

3. WordCraft, GridCraft, DeckCraft, PdfCraft를 별도로 설치했다면, ToolBox의 각 모듈 **설정**에서 실행 파일 경로를 선택할 수 있습니다. PATH에 실행 파일이 이미 등록되어 있으면 자동 탐색합니다.
4. 파일을 선택하고 **선택 파일을 편집기로 열기**를 누릅니다.

**빌드 검증:** GitHub Actions의 Windows 워크플로가 Rust 단위 테스트 및 실행 파일 생성을 담당합니다. 워크플로 성공 여부는 [Actions](https://github.com/cys123431-ship-it/ToolBox/actions)에서 확인할 수 있습니다. 성공한 실행의 Artifacts에 exe가 올라옵니다. 별도 상용 코드 서명은 적용되지 않았습니다.

## 지향점

하나의 앱에서 문서(DOCX/HWPX), 스프레드시트(XLSX), 슬라이드(PPTX), PDF를 **실제로 편집하고 AI가 사용자의 승인을 받아 작업**하는 것입니다. 우선 Windows 11 데스크톱을 구현합니다. 웹 버전은 현재 범위에 포함하지 않습니다.

- [아키텍처](docs/ARCHITECTURE.md)
- [단계별 완료 조건](docs/ROADMAP.md)
- [테스트·제약 사항](docs/TESTING.md)
- [라이선스·서드파티 방침](docs/LICENSES.md)
- [보안 방침](SECURITY.md)

## 참고하는 오픈소스

- [WordCraft](https://github.com/storytold/wordcraft) — DOCX 편집 엔진 후보
- [GridCraft](https://github.com/storytold/gridcraft) — XLSX 엔진 후보
- [DeckCraft](https://github.com/storytold/deckcraft) — PPTX 엔진 후보
- [rHWP](https://github.com/edwardkim/rhwp) — HWP/HWPX 엔진 후보
- [PdfCraft](https://github.com/storytold/pdfcraft) — PDF 엔진 후보

현재 이 프로젝트는 해당 저장소의 소스코드를 복사·번들링하지 않습니다. 독립 실행 편집기들을 **사용자가 별도로 설치**하고 경로를 지정하는 방식입니다. 통합 전에는 각 프로젝트 라이선스와 UI/폰트/브랜드 자산 조건을 다시 점검합니다.

**AI 로그인은 아직 없습니다.** Dockit 소스코드를 사용하지 않으며, 추후 OpenAI의 공식 규격과 이용 조건을 검토한 후 자체 구현합니다.

## 라이선스

ToolBox가 직접 작성한 코드는 MIT 라이선스입니다. 타사 코드·리소스는 포함되는 시점에 해당 라이선스를 별도로 고지합니다. 자세한 내용은 [LICENSE](LICENSE)와 [라이선스 정책](docs/LICENSES.md)을 참조하세요.
