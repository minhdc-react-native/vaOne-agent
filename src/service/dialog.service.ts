import { emit } from "@tauri-apps/api/event";
import { DialogPayload } from "../components/GlobalDialog";

type DialogType = "info" | "success" | "error" | "warning" | "question";

const listeners: Map<string, (result: boolean) => void> = new Map();

function uuid() {
    return Math.random().toString(36).substring(2);
}

export const dialog = {
    open(payload: Omit<DialogPayload, "id">) {
        const id = uuid();

        return new Promise<boolean>((resolve) => {
            listeners.set(id, resolve);

            emit("dialog:open", {
                id,
                ...payload,
            });
        });
    },

    resolve(id: string, result: boolean) {
        const cb = listeners.get(id);
        if (cb) cb(result);
        listeners.delete(id);
    },

    // shortcuts
    info(message: string, title = "Info",
        text?: {
            confirmText?: string;
            cancelText?: string;
        }
    ) {
        return dialog.open({ type: "info", message, title, text });
    },

    success(message: string, title = "Success",
        text?: {
            confirmText?: string;
            cancelText?: string;
        }
    ) {
        return dialog.open({ type: "success", message, title, text });
    },

    error(message: string, title = "Error",
        text?: {
            confirmText?: string;
            cancelText?: string;
        }
    ) {
        return dialog.open({ type: "error", message, title, text });
    },

    warning(message: string, title = "Warning",
        text?: {
            confirmText?: string;
            cancelText?: string;
        }
    ) {
        return dialog.open({ type: "warning", message, title, text });
    },

    question(message: string, title = "Confirm",
        text?: {
            confirmText?: string;
            cancelText?: string;
        }
    ) {
        return dialog.open({ type: "question", message, title, text });
    },
};