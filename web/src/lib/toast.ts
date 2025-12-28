import { createSignal } from "solid-js";

export type ToastType = "success" | "error" | "info" | "warning";

export interface ToastData {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;
}

const [toasts, setToasts] = createSignal<ToastData[]>([]);

const addToast = (type: ToastType, message: string, duration = 5000) => {
  const id = Math.random().toString(36).substring(2, 9);
  setToasts((prev) => [...prev, { id, type, message, duration }]);

  if (duration > 0) {
    setTimeout(() => {
      removeToast(id);
    }, duration);
  }
};

const removeToast = (id: string) => {
  setToasts((prev) => prev.filter((t) => t.id !== id));
};

export const toast = {
  success: (message: string, duration?: number) => addToast("success", message, duration),
  error: (message: string, duration?: number) => addToast("error", message, duration),
  info: (message: string, duration?: number) => addToast("info", message, duration),
  warning: (message: string, duration?: number) => addToast("warning", message, duration),
  remove: removeToast,
};

export { toasts };
