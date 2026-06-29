import { emit, listen } from "@tauri-apps/api/event";

type DialogType = "info" | "success" | "error" | "warning" | "question";

type DialogPayload = {
    id: string;
    type: DialogType;
    title?: string;
    message?: string;
};

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
    info(message: string, title = "Info") {
        return dialog.open({ type: "info", message, title });
    },

    success(message: string, title = "Success") {
        return dialog.open({ type: "success", message, title });
    },

    error(message: string, title = "Error") {
        return dialog.open({ type: "error", message, title });
    },

    warning(message: string, title = "Warning") {
        return dialog.open({ type: "warning", message, title });
    },

    question(message: string, title = "Confirm") {
        return dialog.open({ type: "question", message, title });
    },
};