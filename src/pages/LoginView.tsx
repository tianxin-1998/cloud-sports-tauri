import { useEffect, useState } from "react";
import { useApp } from "../store";
import { login } from "../services";
import { clearLoginInvalid, saveCredential } from "../utils/credentials";
import Dropdown from "../components/Dropdown";

const LS_ACCT = "yunsport_acct";
const LS_REMEMBER = "yunsport_remember";

function BrandMark() {
  return (
    <svg viewBox="0 0 48 48" width="46" height="46" aria-hidden>

      <path d="M6 18h9M4 26h9M7 34h8" stroke="rgba(255,255,255,.55)" strokeWidth="2.6" strokeLinecap="round" fill="none" />

      <circle cx="29.5" cy="12.5" r="4.2" fill="#fff" />
      <path
        d="M26.2 19.2c1.8-1 4.1-.6 5.4.9l3.4 3.9 5.6 1.4a1.9 1.9 0 0 1-.9 3.7l-6.6-1.6a3.6 3.6 0 0 1-1.7-1l-1.4-1.5-1.5 5.2 4.3 4.6a2.6 2.6 0 0 1 .6 1.2l1.3 6.2a2 2 0 0 1-3.9.8l-1.2-5.5-5.3-5.6a3 3 0 0 1-.7-2.9l1.6-5.6-2.9 1.4-1.9 4.4a1.9 1.9 0 0 1-3.5-1.5l2.2-5.2a3.4 3.4 0 0 1 1.7-1.7l5.5-2.7z"
        fill="#fff"
      />
    </svg>
  );
}

const IconSchool = (
  <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" strokeWidth="1.9" strokeLinecap="round" strokeLinejoin="round">
    <path d="M3 9.5 12 4l9 5.5-9 5.5-9-5.5Z" />
    <path d="M7 12v4.6c0 .8 2.2 2.4 5 2.4s5-1.6 5-2.4V12" />
  </svg>
);
const IconUser = (
  <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" strokeWidth="1.9" strokeLinecap="round" strokeLinejoin="round">
    <circle cx="12" cy="8" r="3.6" />
    <path d="M4.5 20c1.4-3.3 4.2-5 7.5-5s6.1 1.7 7.5 5" />
  </svg>
);
const IconLock = (
  <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" strokeWidth="1.9" strokeLinecap="round" strokeLinejoin="round">
    <rect x="5" y="10.5" width="14" height="9.5" rx="2.5" />
    <path d="M8.5 10.5V8a3.5 3.5 0 0 1 7 0v2.5" />
  </svg>
);
const IconErr = (
  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
    <circle cx="12" cy="12" r="9" />
    <path d="M12 7.5v5.5M12 16.6v.1" />
  </svg>
);

export default function LoginView() {
  const { schools, cfg, setSession, show, session, authHint, relogin, setRelogin } = useApp();
  const [school, setSchool] = useState(cfg.default_school_code || "");
  const [user, setUser] = useState("");
  const [pass, setPass] = useState("");
  const [remember, setRemember] = useState(true);
  const [err, setErr] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {

    if (relogin) {
      if (relogin.user) setUser(relogin.user);
      if (relogin.password) setPass(relogin.password);
      if (relogin.school_code) setSchool(relogin.school_code);
      return;
    }
    try {
      const r = localStorage.getItem(LS_REMEMBER);
      if (r !== null) setRemember(r === "1");
      const acct = JSON.parse(localStorage.getItem(LS_ACCT) || "null");
      if (acct) {
        if (acct.user) setUser(acct.user);
        if (acct.password) setPass(acct.password);
        if (acct.school_code) setSchool(acct.school_code);
      } else if (cfg.default_school_code) {
        setSchool(cfg.default_school_code);
      }
    } catch {

    }

  }, []);

  const schoolLabel = schools.length ? `选择学校（${schools.length} 所）` : "加载学校列表…";
  const options = schools.map((s) => ({ value: s.schoolCode, label: s.schoolName, sub: s.schoolCode }));

  const persistRemember = (() => {
    try {
      localStorage.setItem(LS_REMEMBER, remember ? "1" : "0");
    } catch {

    }
  });

  const doLogin = async () => {
    setErr("");
    if (!school) return setErr("请选择学校");
    if (!user.trim() || !pass) return setErr("请填写学号和密码");
    setBusy(true);
    try {
      const d = await login({ user: user.trim(), password: pass, school_code: school });

      saveCredential({ user: user.trim(), password: pass, school_code: school });
      clearLoginInvalid(user.trim(), school);
      setRelogin(null);
      persistRemember();
      try {
        if (remember) {
          localStorage.setItem(LS_ACCT, JSON.stringify({ user: user.trim(), password: pass, school_code: school }));
        } else {
          localStorage.removeItem(LS_ACCT);
        }
      } catch {

      }
      setSession(d);
      show("home");
    } catch (e: any) {
      console.error("[login] 失败:", e);
      setErr(String(e?.message ?? e));
    } finally {
      setBusy(false);
    }
  };

  const onKey = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !busy) doLogin();
  };

  return (
    <div className="view v-login on" id="view-login">
      <span className="lg-orb lg-orb1" />
      <span className="lg-orb lg-orb2" />

      <div className="lg-brand">
        <div className="lg-logo">
          <BrandMark />
          <span className="lg-glow" />
        </div>
        <h2>云运动助手</h2>
        <p className="sub">校园跑步好帮手</p>
      </div>

      {authHint && (
        <div className="lg-auth-hint">
          <span className="lg-auth-dot" />
          {authHint}
        </div>
      )}

      <div className="lg-card">
        <div className="lg-input">
          <i>{IconSchool}</i>
          <Dropdown
            options={options}
            value={school}
            onChange={setSchool}
            placeholder={schoolLabel}
            searchable
            searchPlaceholder="搜索学校名称或编码…"
          />
        </div>
        <div className="lg-input">
          <i>{IconUser}</i>
          <input placeholder="学号" autoComplete="username" value={user} onChange={(e) => setUser(e.target.value)} onKeyDown={onKey} />
        </div>
        <div className="lg-input">
          <i>{IconLock}</i>
          <input placeholder="密码" type="password" autoComplete="current-password" value={pass} onChange={(e) => setPass(e.target.value)} onKeyDown={onKey} />
        </div>

        <div className="lg-row">
          <button type="button" className={"lg-switch" + (remember ? " on" : "")} role="switch" aria-checked={remember} onClick={() => setRemember((v) => !v)}>
            <span />
          </button>
          <span className="lg-switch-label" onClick={() => setRemember((v) => !v)}>
            记住账号密码
          </span>
          <em className="lg-safe">数据仅保存在本机</em>
        </div>

        <button className="m-btn lg-submit" disabled={busy} onClick={doLogin}>
          {busy ? (
            <>
              <span className="lg-spin" />
              登录中…
            </>
          ) : (
            <>
              登录
              <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M5 12h13M13 6.5 19 12l-6 5.5" />
              </svg>
            </>
          )}
        </button>

        {err && (
          <p className="lg-err">
            {IconErr}
            {err}
          </p>
        )}
      </div>

      {session && (
        <button
          className="m-btn sm"
          style={{ marginTop: 12, background: "rgba(255,255,255,.75)", color: "var(--ink)" }}
          onClick={() => {

            setRelogin(null);
            show("set");
          }}
        >
          取消添加，返回
        </button>
      )}

      <p className="lg-foot">此应用只用于学习交流使用</p>
    </div>
  );
}
