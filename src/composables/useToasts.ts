import { readonly, ref } from 'vue';

export type ToastKind = 'error' | 'success' | 'info';

export interface Toast {
  id: number;
  message: string;
  kind: ToastKind;
}

const toasts = ref<Toast[]>([]);
let nextToastId = 1;

export function notify(message: unknown, kind: ToastKind = 'error') {
  const id = nextToastId++;
  toasts.value.push({ id, message: String(message), kind });
  window.setTimeout(() => dismissToast(id), kind === 'error' ? 8000 : 4500);
  return id;
}

export function dismissToast(id: number) {
  toasts.value = toasts.value.filter(toast => toast.id !== id);
}

export function useToasts() {
  return { toasts: readonly(toasts), dismissToast };
}
