# ShotAuto - 개발 진행 상황

## ✅ 완료된 작업

### Phase 1: 기본 앱 셋업
- [x] **Rust 1.92.0 설치** - rustup으로 설치 완료
- [x] **Tauri + React 프로젝트 생성** - `create-tauri-app` 템플릿
- [x] **SQLite 스키마 작성** (`src-tauri/src/db.rs`)
  - `config` - API 키 저장
  - `trends` - YouTube 트렌드 데이터
  - `jobs` - 작업 큐
  - `shorts` - 생성된 영상
  - `metrics` - 성능 로그
- [x] **Tauri 백엔드 명령어** (`src-tauri/src/lib.rs`)
  - `get_config` / `save_config`
  - `get_stats`
  - `test_youtube_api` / `test_telegram_bot` / `test_ollama`
- [x] **React 프론트엔드 UI** (`src/App.tsx`, `src/App.css`)
  - 대시보드 (통계 카드)
  - 설정 화면 (API 키 입력/테스트)
  - 모던 다크 테마

---

## ⏸️ 중지된 작업

### Visual Studio Build Tools 설치 대기
**원인**: Rust MSVC 타겟 컴파일에 `link.exe` 링커 필요

**해결 방법**:
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --quiet --wait"
```

설치 완료 후:
```powershell
cd src-tauri
cargo build
```

---

## 📂 프로젝트 구조

```
shotauto/
├── src/                    # React 프론트엔드
│   ├── App.tsx            # 메인 앱 (대시보드 + 설정)
│   └── App.css            # 다크 테마 스타일
├── src-tauri/             # Rust 백엔드
│   ├── Cargo.toml         # 의존성 (rusqlite, reqwest, tokio 등)
│   └── src/
│       ├── lib.rs         # Tauri 명령어
│       ├── db.rs          # SQLite 모듈
│       └── main.rs        # 진입점
└── README.md              # 아키텍처 문서
```

---

## 🚀 다음 단계

1. VS Build Tools 설치 완료 후 `cargo build` 재시도
2. `npm run tauri dev`로 개발 서버 실행
3. Phase 2: YouTube API 트렌드 수집 구현
4. Phase 3: Telegram 알림 기능
5. Phase 4: Windows 인스톨러 패키징
