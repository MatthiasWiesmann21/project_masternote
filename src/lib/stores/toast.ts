import { writable } from 'svelte/store';

export type ToastType = 'success' | 'error' | 'info' | 'warning';

export interface Toast {
  id: number;
  type: ToastType;
  message: string;
  duration: number;
}

let nextId = 0;

export const toasts = writable<Toast[]>([]);

export function showToast(message: string, type: ToastType = 'info', duration = 3000) {
  const id = ++nextId;
  toasts.update((list) => [...list, { id, type, message, duration }]);
  if (duration > 0) {
    setTimeout(() => dismissToast(id), duration);
  }
}

export function dismissToast(id: number) {
  toasts.update((list) => list.filter((t) => t.id !== id));
}
