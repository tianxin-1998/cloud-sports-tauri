import { useEffect, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { RunProgressPayload } from "../types";

const RUN_PROGRESS_EVENT = "run-progress";

export function useRunProgress(onPayload: (payload: RunProgressPayload) => void): void {
  const cbRef = useRef(onPayload);
  cbRef.current = onPayload;

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;

    listen<RunProgressPayload>(RUN_PROGRESS_EVENT, (e) => cbRef.current(e.payload)).then((fn) => {

      if (disposed) fn();
      else unlisten = fn;
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}
