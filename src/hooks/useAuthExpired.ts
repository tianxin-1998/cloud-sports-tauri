import { useEffect, useRef } from "react";
import { onAuthExpired } from "../services";

const noop = () => {};

export function useAuthExpired(handler: (msg: string) => void): void {
  const cbRef = useRef(handler);
  cbRef.current = handler;

  useEffect(() => {
    onAuthExpired((msg) => cbRef.current(msg));
    return () => onAuthExpired(noop);
  }, []);
}
