import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Config {
  youtube_api_key: string | null;
  telegram_bot_token: string | null;
  telegram_chat_id: string | null;
  ollama_endpoint: string;
  poll_interval_secs: number;
}

interface DashboardStats {
  total_trends: number;
  pending_jobs: number;
  completed_jobs: number;
  failed_jobs: number;
}

type TabType = "dashboard" | "settings";

function App() {
  const [tab, setTab] = useState<TabType>("dashboard");
  const [config, setConfig] = useState<Config>({
    youtube_api_key: null,
    telegram_bot_token: null,
    telegram_chat_id: null,
    ollama_endpoint: "http://localhost:11434",
    poll_interval_secs: 300,
  });
  const [stats, setStats] = useState<DashboardStats>({
    total_trends: 0,
    pending_jobs: 0,
    completed_jobs: 0,
    failed_jobs: 0,
  });
  const [isRunning, setIsRunning] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const [testing, setTesting] = useState<string | null>(null);

  useEffect(() => {
    loadConfig();
    loadStats();
  }, []);

  const loadConfig = async () => {
    try {
      const cfg = await invoke<Config>("get_config");
      setConfig(cfg);
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  };

  const loadStats = async () => {
    try {
      const s = await invoke<DashboardStats>("get_stats");
      setStats(s);
    } catch (e) {
      console.error("Failed to load stats:", e);
    }
  };

  const saveConfig = async () => {
    try {
      await invoke("save_config", { config });
      showToast("✅ 설정이 저장되었습니다");
    } catch (e) {
      showToast("❌ 저장 실패: " + e);
    }
  };

  const testApi = async (type: "youtube" | "telegram" | "ollama") => {
    setTesting(type);
    try {
      let result = false;
      switch (type) {
        case "youtube":
          if (config.youtube_api_key) {
            result = await invoke<boolean>("test_youtube_api", { apiKey: config.youtube_api_key });
          }
          break;
        case "telegram":
          if (config.telegram_bot_token) {
            result = await invoke<boolean>("test_telegram_bot", { token: config.telegram_bot_token });
          }
          break;
        case "ollama":
          result = await invoke<boolean>("test_ollama", { endpoint: config.ollama_endpoint });
          break;
      }
      showToast(result ? "✅ 연결 성공!" : "❌ 연결 실패");
    } catch (e) {
      showToast("❌ 테스트 실패: " + e);
    } finally {
      setTesting(null);
    }
  };

  const showToast = (message: string) => {
    setToast(message);
    setTimeout(() => setToast(null), 3000);
  };

  const updateConfig = (key: keyof Config, value: string | number) => {
    setConfig((prev) => ({ ...prev, [key]: value || null }));
  };

  return (
    <div className="app">
      <header className="header">
        <h1>🎬 ShotAuto</h1>
        <div className="header-status">
          <span className={`status-dot ${isRunning ? "" : "stopped"}`}></span>
          <span>{isRunning ? "실행 중" : "정지됨"}</span>
        </div>
      </header>

      <nav className="nav">
        <button
          className={`nav-btn ${tab === "dashboard" ? "active" : ""}`}
          onClick={() => setTab("dashboard")}
        >
          📊 대시보드
        </button>
        <button
          className={`nav-btn ${tab === "settings" ? "active" : ""}`}
          onClick={() => setTab("settings")}
        >
          ⚙️ 설정
        </button>
      </nav>

      <main className="main">
        {tab === "dashboard" && (
          <>
            <div className="stats-grid">
              <div className="stat-card">
                <div className="stat-value">{stats.total_trends}</div>
                <div className="stat-label">수집된 트렌드</div>
              </div>
              <div className="stat-card">
                <div className="stat-value">{stats.completed_jobs}</div>
                <div className="stat-label">생성된 영상</div>
              </div>
              <div className="stat-card">
                <div className="stat-value">{stats.pending_jobs}</div>
                <div className="stat-label">대기 중</div>
              </div>
              <div className="stat-card">
                <div className="stat-value">{stats.failed_jobs}</div>
                <div className="stat-label">실패</div>
              </div>
            </div>

            <div className="control-bar">
              <button
                className={`btn ${isRunning ? "btn-secondary" : "btn-success"}`}
                onClick={() => setIsRunning(!isRunning)}
              >
                {isRunning ? "⏸️ 일시정지" : "▶️ 시작"}
              </button>
              <button className="btn btn-secondary" onClick={loadStats}>
                🔄 새로고침
              </button>
            </div>
          </>
        )}

        {tab === "settings" && (
          <>
            {/* YouTube API */}
            <div className="card">
              <div className="card-header">
                <div className="card-icon youtube">📺</div>
                <h2>YouTube API</h2>
              </div>
              <div className="form-group">
                <label className="form-label">API Key</label>
                <div className="input-row">
                  <input
                    type="password"
                    className="form-input"
                    placeholder="AIzaSy..."
                    value={config.youtube_api_key || ""}
                    onChange={(e) => updateConfig("youtube_api_key", e.target.value)}
                  />
                  <button
                    className="btn btn-secondary btn-small"
                    onClick={() => testApi("youtube")}
                    disabled={testing === "youtube" || !config.youtube_api_key}
                  >
                    {testing === "youtube" ? <span className="spinner"></span> : "테스트"}
                  </button>
                </div>
              </div>
            </div>

            {/* Telegram */}
            <div className="card">
              <div className="card-header">
                <div className="card-icon telegram">✈️</div>
                <h2>Telegram Bot</h2>
              </div>
              <div className="form-group">
                <label className="form-label">Bot Token</label>
                <div className="input-row">
                  <input
                    type="password"
                    className="form-input"
                    placeholder="123456:ABC-DEF..."
                    value={config.telegram_bot_token || ""}
                    onChange={(e) => updateConfig("telegram_bot_token", e.target.value)}
                  />
                  <button
                    className="btn btn-secondary btn-small"
                    onClick={() => testApi("telegram")}
                    disabled={testing === "telegram" || !config.telegram_bot_token}
                  >
                    {testing === "telegram" ? <span className="spinner"></span> : "테스트"}
                  </button>
                </div>
              </div>
              <div className="form-group">
                <label className="form-label">Chat ID</label>
                <input
                  type="text"
                  className="form-input"
                  placeholder="-1001234567890"
                  value={config.telegram_chat_id || ""}
                  onChange={(e) => updateConfig("telegram_chat_id", e.target.value)}
                />
              </div>
            </div>

            {/* Ollama */}
            <div className="card">
              <div className="card-header">
                <div className="card-icon ollama">🤖</div>
                <h2>Ollama</h2>
              </div>
              <div className="form-group">
                <label className="form-label">Endpoint</label>
                <div className="input-row">
                  <input
                    type="text"
                    className="form-input"
                    placeholder="http://localhost:11434"
                    value={config.ollama_endpoint}
                    onChange={(e) => updateConfig("ollama_endpoint", e.target.value)}
                  />
                  <button
                    className="btn btn-secondary btn-small"
                    onClick={() => testApi("ollama")}
                    disabled={testing === "ollama"}
                  >
                    {testing === "ollama" ? <span className="spinner"></span> : "테스트"}
                  </button>
                </div>
              </div>
            </div>

            {/* Save Button */}
            <button className="btn btn-primary" onClick={saveConfig} style={{ width: "100%" }}>
              💾 설정 저장
            </button>
          </>
        )}
      </main>

      {toast && <div className="toast">{toast}</div>}
    </div>
  );
}

export default App;
