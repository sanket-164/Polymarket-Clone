"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";

type ToastTone = "success" | "error" | "info";

type Toast = {
  id: number;
  message: string;
  tone: ToastTone;
};

type ToastContextValue = {
  showToast: (message: string, tone?: ToastTone) => void;
  success: (message: string) => void;
  error: (message: string) => void;
  info: (message: string) => void;
};

const ToastContext = createContext<ToastContextValue | null>(null);
let nextToastId = 0;

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const showToast = useCallback((message: string, tone: ToastTone = "info") => {
    const id = nextToastId++;
    setToasts((currentToasts) =>
      [...currentToasts, { id, message, tone }].slice(-4)
    );
  }, []);

  const dismissToast = useCallback((id: number) => {
    setToasts((currentToasts) =>
      currentToasts.filter((toast) => toast.id !== id)
    );
  }, []);

  useEffect(() => {
    if (toasts.length === 0) {
      return;
    }

    const timers = toasts.map((toast) =>
      window.setTimeout(() => dismissToast(toast.id), 4500)
    );

    return () => timers.forEach((timer) => window.clearTimeout(timer));
  }, [dismissToast, toasts]);

  const value = useMemo<ToastContextValue>(
    () => ({
      showToast,
      success: (message) => showToast(message, "success"),
      error: (message) => showToast(message, "error"),
      info: (message) => showToast(message, "info"),
    }),
    [showToast]
  );

  return (
    <ToastContext.Provider value={value}>
      {children}
      <div
        className="pointer-events-none fixed inset-x-4 top-[88px] z-[70] flex flex-col items-end gap-3 sm:left-auto sm:w-[min(380px,calc(100vw-2rem))]"
        aria-live="polite"
        aria-atomic="true"
      >
        {toasts.map((toast) => (
          <div
            key={toast.id}
            role={toast.tone === "error" ? "alert" : "status"}
            className={`pointer-events-auto flex w-full items-start gap-3 rounded-xl border px-4 py-3 text-sm shadow-2xl ${getToastClassName(toast.tone)}`}
          >
            <span className="flex-1 leading-5">{toast.message}</span>
            <button
              type="button"
              onClick={() => dismissToast(toast.id)}
              aria-label="Dismiss notification"
              className="shrink-0 text-lg leading-5 text-text/75 transition hover:text-text"
            >
              ×
            </button>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}

export function useToast() {
  const context = useContext(ToastContext);

  if (!context) {
    throw new Error("useToast must be used within ToastProvider");
  }

  return context;
}

function getToastClassName(tone: ToastTone) {
  if (tone === "success") {
    return "border-buy bg-buy text-text";
  }

  if (tone === "error") {
    return "border-sell bg-sell text-text";
  }

  return "border-accent bg-accent text-text";
}
