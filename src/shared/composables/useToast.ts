import { ref } from "vue";

export type ToastType = "success" | "error" | "info";

export type ToastItem = {
  id: number;
  type: ToastType;
  message: string;
};

const toasts = ref<ToastItem[]>([]);
let nextId = 0;

function show(type: ToastType, message: string, duration = 4000) {
  const id = nextId++;
  toasts.value = [...toasts.value, { id, type, message }];
  setTimeout(() => dismiss(id), duration);
}

function dismiss(id: number) {
  toasts.value = toasts.value.filter((t) => t.id !== id);
}

export function useToast() {
  return {
    toasts,
    success: (msg: string) => show("success", msg),
    error: (msg: string) => show("error", msg),
    info: (msg: string) => show("info", msg),
    dismiss,
  };
}
