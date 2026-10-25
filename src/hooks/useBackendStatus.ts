import { useEffect, useState } from "react";
import { status } from "../services";

export interface BackendStatus {
  on: boolean;
  text: string;
}

export function useBackendStatus(intervalMs = 15000): BackendStatus {
  const [conn, setConn] = useState<BackendStatus>({ on: false, text: "检测中" });

  useEffect(() => {
    let alive = true;
    const check = async () => {
      try {
        await status();
        if (alive) setConn({ on: true, text: "正常" });
      } catch {
        if (alive) setConn({ on: false, text: "异常" });
      }
    };
    check();
    const timer = window.setInterval(check, intervalMs);
    return () => {
      alive = false;
      window.clearInterval(timer);
    };
  }, [intervalMs]);

  return conn;
}
