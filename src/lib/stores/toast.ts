// ============================================================================
// Keepix — Toast Notification Store
// ============================================================================

import { writable } from 'svelte/store';

export interface Toast {
  id: string;
  message: string;
  type: 'info' | 'success' | 'warning' | 'error';
  duration: number;
  undoAction?: () => void;
}

export const toasts = writable<Toast[]>([]);

let counter = 0;

/** Show a toast notification */
export function showToast(
  message: string,
  type: Toast['type'] = 'info',
  duration: number = 2500,
  undoAction?: () => void
) {
  const id = `toast-${++counter}`;
  const toast: Toast = { id, message, type, duration, undoAction };

  toasts.update(t => [...t, toast]);

  // Auto-dismiss
  setTimeout(() => {
    dismissToast(id);
  }, duration);

  return id;
}

/** Dismiss a specific toast */
export function dismissToast(id: string) {
  toasts.update(t => t.filter(toast => toast.id !== id));
}

/** Convenience methods */
export const toast = {
  info: (msg: string, duration?: number) => showToast(msg, 'info', duration),
  success: (msg: string, duration?: number) => showToast(msg, 'success', duration),
  warning: (msg: string, duration?: number) => showToast(msg, 'warning', duration),
  error: (msg: string, duration?: number) => showToast(msg, 'error', duration),
  undo: (msg: string, undoFn: () => void) => showToast(msg, 'success', 4000, undoFn),
};
