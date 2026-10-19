# CSV Edit Plan v1

사용자가 ToolBox 내부 CSV/TSV 편집기의 "JSON 변경 계획" 영역에 다음 JSON을 붙여넣는다.

```json
{
  "version": 1,
  "source_sha256": "<ToolBox 화면에 표시된 현재 원본 파일 SHA-256 64자리>",
  "changes": [
    {"row": 1, "col": 1, "expected": "80", "replacement": "85"},
    {"row": 2, "col": 1, "expected": "90", "replacement": "95"}
  ]
}
```

- row, col: 0부터 시작하는 **정수 인덱스**. 0행은 일반적으로 헤더 행.
- expected: 현재 셀 값. 공백이나 줄바꿈도 정확히 일치해야 한다.
- replacement: 승인 후 적용할 새로운 문자열. 수식 실행 기능이 아니다.
- source_sha256: 파일을 열거나 마지막 저장한 시점의 원본 바이트 SHA-256.
- 최대 JSON 256 KiB, 변경 셀 1~200개, 결과 표 2 MiB.
- 버전 1 외의 형식 및 알 수 없는 필드는 거부.
- 미리보기 후 변경 발생 시 재검토 필요.
- 사용자 승인 없이 변경 적용 금지. 메모리 적용과 디스크 저장은 별도 작업.

이 형식은 **로컬 데이터 변경 제안**만 나타내며 AI 명령 실행·권한 부여·인증 기능을 포함하지 않는다.
