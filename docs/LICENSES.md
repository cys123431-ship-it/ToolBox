# 라이선스 및 출처 정책

## 자체 코드

ToolBox의 신규 소스는 MIT 라이선스. LICENSE를 참조한다.

## 외부 소스

**현재 해당 프로젝트들의 코드·아이콘·폰트를 저장소에 복사하지 않았다.** 사용자가 별도로 설치한 편집기의 실행 파일을 호출한다.

검토 대상:
- Storytold/WordCraft: MIT OR Apache-2.0 (공개 README 및 LICENSE-MIT/LICENSE-APACHE 참조)
- Storytold/GridCraft: MIT OR Apache-2.0
- Storytold/DeckCraft: MIT OR Apache-2.0
- Storytold/PdfCraft: MIT OR Apache-2.0
- edwardkim/rhwp: MIT
- nokryong/Dockit-open: AGPL-3.0-only — 사용하지 않음.

각 프로젝트의 저장소 메타데이터 표시뿐 아니라 LICENSE 파일, NOTICE, ATTRIBUTION 및 모든 서드파티 자산의 조건을 확인한 다음 직접 번들링 여부를 결정한다. **MIT/Apache도 적절한 고지 의무가 있다.** Storytold의 프로젝트 브랜드·로고는 그대로 가져오지 않는다.

## 현재 직접 사용 중인 Rust 의존성

csv, eframe, rfd, serde, serde_json, tempfile은 crates.io에서 빌드 시 설치한다. 실제 번들에 포함된 모든 전이 의존성과 라이선스는 Cargo.lock 생성 후 별도로 확인·고지한다. 자동 검증과 릴리스 시 라이선스 목록 생성 작업을 추가할 예정이다. 서드파티 라이브러리의 라이선스가 이 프로젝트의 LICENSE로 변경되는 것은 아니다.

## AI 로그인

OpenAI SIWC·Codex 코드 구현 전 공식 규격, 도구 사용 정책, 인증 저장 방법, 배포 조건을 다시 점검한다. Dockit의 구현 코드는 사용하지 않는다. 타인의 비공개 인증 토큰, 쿠키, 세션을 가져오지 않는다.
