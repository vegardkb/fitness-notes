import { writable } from "svelte/store";

type ToastLevel = "success" | "error";
type ToastState = { message: string; level: ToastLevel } | null;

function createToast() {
    const { subscribe, set } = writable<ToastState>(null);
    let timer: ReturnType<typeof setTimeout> | undefined;

    return {
        subscribe,
        show(message: string, level: ToastLevel = "success") {
            clearTimeout(timer);
            set({ message, level });
            timer = setTimeout(() => set(null), 3000);
        },
        dismiss() {
            clearTimeout(timer);
            set(null);
        },
    };
}

export const toast = createToast();
