import { getCurrentWindow } from "@tauri-apps/api/window";
import { AppProvider, useApp } from "./store";
import { useBackendStatus } from "./hooks";
import ConfirmDialog from "./components/ConfirmDialog";
import LogoMark from "./components/LogoMark";
import LoginView from "./pages/LoginView";
import HomeView from "./pages/HomeView";
import SettingsView from "./pages/SettingsView";
import DrawView from "./pages/DrawView";
import SimView from "./pages/SimView";
import DetailView from "./pages/DetailView";

function Shell() {
  const { view, show, reloginAsk, confirmRelogin, cancelRelogin } = useApp();
  const conn = useBackendStatus();

  const inTauri = typeof window !== "undefined" && !!(window as any).__TAURI_INTERNALS__;

  return (
    <div className="screen">

      <div className="titlebar" data-tauri-drag-region>
        <div className="tb-left" data-tauri-drag-region>
          <LogoMark size={18} />
          <span className="tb-name">云运动助手</span>
          <span className="tb-status">
            <i className={"conn-dot" + (conn.on ? " on" : "")} />
            {conn.text}
          </span>
        </div>
        <div className="tb-btns">
          <button className="tb-btn" aria-label="最小化" onClick={() => inTauri && getCurrentWindow().minimize()}>
            &#x2500;
          </button>
          <button className="tb-btn tb-close" aria-label="关闭" onClick={() => inTauri && getCurrentWindow().close()}>
            &#x00D7;
          </button>
        </div>
      </div>

          <div className="views">
            {view === "login" && <LoginView />}
            {view === "home" && <HomeView />}
            {view === "set" && <SettingsView />}
            {view === "draw" && <DrawView />}
            {view === "detail" && <DetailView />}
            {view === "sim" && <SimView />}
          </div>

          {view !== "login" && view !== "draw" && view !== "detail" && (
          <div className="tabbar">
            <button className={"tab" + (view === "home" ? " on" : "")} onClick={() => show("home")}>
              <svg viewBox="0 0 24 24">
                <path d="M3 11 12 4l9 7" />
                <path d="M6 10v10h12V10" />
              </svg>
              首页
            </button>
            <button className={"tab" + (view === "sim" ? " on" : "")} onClick={() => show("sim")}>
              <svg viewBox="0 0 24 24">
                <path d="M13.49 5.48c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm-3.6 13.9l1-4.4 2.1 2v6h2v-7.5l-2.1-2 .6-3c1.3 1.5 3.3 2.5 5.5 2.5v-2c-1.9 0-3.5-1-4.3-2.4l-1-1.6c-.4-.6-1-1-1.7-1-.3 0-.5.1-.8.1l-5.2 2.2v4.7h2v-3.4l1.8-.7-1.6 8.1-4.9-1-.4 2 7 1.4z" />
              </svg>
              跑步助手
            </button>
            <button className={"tab" + (view === "set" ? " on" : "")} onClick={() => show("set")}>
              <svg viewBox="0 0 24 24">
                <circle cx="12" cy="12" r="3" />
                <path d="M19.4 15a1.7 1.7 0 0 0 .34 1.87l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.7 1.7 0 0 0-1.87-.34 1.7 1.7 0 0 0-1 1.55V21a2 2 0 1 1-4 0v-.09a1.7 1.7 0 0 0-1-1.55 1.7 1.7 0 0 0-1.87.34l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.7 1.7 0 0 0 .34-1.87 1.7 1.7 0 0 0-1.55-1H3a2 2 0 1 1 0-4h.09a1.7 1.7 0 0 0 1.55-1 1.7 1.7 0 0 0-.34-1.87l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.7 1.7 0 0 0 1.87.34h.01a1.7 1.7 0 0 0 1-1.55V3a2 2 0 1 1 4 0v.09a1.7 1.7 0 0 0 1 1.55 1.7 1.7 0 0 0 1.87-.34l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.7 1.7 0 0 0-.34 1.87v.01a1.7 1.7 0 0 0 1.55 1H21a2 2 0 1 1 0 4h-.09a1.7 1.7 0 0 0-1.55 1Z" />
              </svg>
              设置
            </button>
          </div>
          )}

        <ConfirmDialog
          open={!!reloginAsk}
          title="账号登录已失效"
          message={
            reloginAsk
              ? `${reloginAsk.name}（${reloginAsk.user}）的登录状态已失效，已用本机保存的密码尝试重新登录仍未成功。是否回到登录页重新登录？`
              : ""
          }
          confirmText="重新登录"
          cancelText="标记失效"
          onConfirm={confirmRelogin}
          onCancel={cancelRelogin}
        />
      </div>
  );
}

export default function App() {
  return (
    <AppProvider>
      <Shell />
    </AppProvider>
  );
}
