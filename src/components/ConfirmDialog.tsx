import { useEffect } from "react";
import { createPortal } from "react-dom";
import type { ReactNode } from "react";

export interface ConfirmDialogProps {
  open: boolean;
  title: string;
  message: ReactNode;
  confirmText?: string;
  cancelText?: string;
  danger?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export default function ConfirmDialog({
  open,
  title,
  message,
  confirmText = "确认",
  cancelText = "取消",
  danger,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onCancel]);

  if (!open) return null;

  return createPortal(
    <div className="dlg-mask" onClick={onCancel}>
      <div className="dlg" role="dialog" aria-modal="true" onClick={(e) => e.stopPropagation()}>
        <div className="dlg-title">{title}</div>
        <div className="dlg-body">{message}</div>
        <div className="dlg-btns">
          <button type="button" className="m-btn sm dlg-cancel" onClick={onCancel}>
            {cancelText}
          </button>
          <button type="button" className={"m-btn sm" + (danger ? " danger" : "")} onClick={onConfirm}>
            {confirmText}
          </button>
        </div>
      </div>
    </div>,
    document.body,
  );
}
