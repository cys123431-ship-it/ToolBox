# ToolBox 아키텍처 (ADR-0001)

## 결론

**우선 Windows 11에 설치해서 실행할 수 있는 Rust 네이티브 데스크톱 앱**을 구축한다. 웹은 후순위이다. 주요 편집기 후보는 Storytold의 WordCraft, GridCraft, DeckCraft, PdfCraft 및 edwardkim/rhwp. Dockit의 소스는 사용하지 않는다.

### 구현 상태

1. [구현] ToolBox UI: egui 앱, 모듈별 뷰, 파일 선택/드롭, 최근 파일.
2. [구현] 엔진 Registry: 파일 형식 분류, PATH/사용자 지정 경로, 안전한 프로세스 시작.
3. [미구현] 엔진 임베드: 독립 앱을 동일 창에서 편집하는 기능.
4. [미구현] rHWP: CLI/WASM 기반 엔진의 데스크톱 브리지.
5. [미구현] OpenAI ChatGPT 계정 로그인과 Codex.
6. [미구현] 복잡한 DOCX/HWPX/XLSX/PPTX의 파일 왕복 보존 검증.

## 최초 선택: Launcher 검증 단계를 의도적으로 분리

독립된 Rust 코드베이스들을 그냥 Cargo workspace에 나란히 넣는다고 통합 편집기가 되지 않는다. 각 앱의 공통 egui 컨텍스트/상태, 문서 탭, 키보드 이벤트, 파일 입출력 및 라이브러리 내부 API를 조사해야 한다. 따라서 별도 실행 가능한 편집기 탐지·파일 라우팅부터 검증한다. 이는 첫 번째 기술 검증 단계일 뿐, 최종 UI 아키텍처는 아니다.

## 단계별 모듈

- Desktop shell: 앱 시작/종료, 네비게이션, 다중 탭(예정), 설정, 오류 표시.
- Workspace core: 열린 파일 상태/잠금/이력/저장/백업/복구 (예정).
- Engine adapters: WordCraft, GridCraft, DeckCraft, rHWP, PdfCraft의 라이브러리/명령 API (현재 외부 실행만 구현).
- AI command gateway: 사용자 동의→권한 범위 지정→작업 계획→변경 미리보기→승인→실행→검증→되돌리기 (예정).
- Export validation: 파일을 저장 후 독립 엔진으로 다시 읽고 내용·페이지·객체를 비교 (예정).

## 어댑터 평가 항목

각 후보별로 (1) Rust API 안정성 (2) 별도 앱 코드 없이 편집 UI 재사용 가능 여부 (3) 파일 읽기/쓰기 일관성 (4) 한글 폰트/IME (5) MCP/CLI 범위 (6) 빌드 크기·성능 (7) 라이선스 및 브랜드 자산을 검토한다.

그 결과에 따라 다음 중 하나를 선택한다.
- A: 개별 편집 엔진/프런트엔드를 라이브러리로 포함해 통합 창 구축.
- B: 엔진만 재사용하고 ToolBox 공통 UI에서 렌더링·편집.
- C: 임시로 독립 실행 방식 유지하고 기능별 호환성/자동화 검증을 우선.

구체적 API가 검증되기 전 특정 임베드 방식이 가능하다고 단정하지 않는다.

## 로컬 데이터

설정: Windows %APPDATA%/ToolBox/preferences.json (다른 OS는 XDG_CONFIG_HOME 또는 ~/.config/ToolBox). 최근 파일 경로와 선택한 외부 편집기 실행 경로만 포함한다. 계정 토큰, 문서 내용, API 키는 현재 저장하지 않는다.

## 보안

- 파일 실행 시 shell/cmd/powershell 문자열을 생성하지 않는다. Command::new(binary).arg(document).
- 편집기는 사용자 지정 경로 또는 PATH에서 실제 파일일 때만 실행한다.
- 기본 값으로 사용자 디렉터리 전체 접근 권한을 가진 AI 에이전트를 실행하지 않는다.
- 향후 자동 편집은 사용자 선택 워크스페이스 디렉터리로 읽기/쓰기 범위를 제한한다.
- 문서가 AI로 전달되는 범위와 온라인 송신 여부를 UI에서 명확하게 고지한다.

## 기타

Windows에서 한글 문자는 설치된 맑은 고딕을 앱의 egui 폰트로 로드한다. 폰트 자체를 저장소에 포함하지 않는다.
