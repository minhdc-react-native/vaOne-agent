import { emit } from "@tauri-apps/api/event";
import { DialogPayload } from "../components/GlobalDialog";
import { ReactNode } from "react";

type DialogType = "info" | "success" | "error" | "warning" | "question";

export const contents = new Map<string, ReactNode>();

export const listeners: Map<string, (result: boolean) => void> = new Map();

function uuid() {
    return Math.random().toString(36).substring(2);
}

export const dialog = {
    open(payload: Omit<DialogPayload, "id">) {
        const id = uuid();

        return new Promise<boolean>((resolve) => {
            listeners.set(id, resolve);

            if (payload.content !== undefined) {
                contents.set(id, payload.content);
            }

            const { content, ...eventPayload } = payload;

            emit("dialog:open", {
                id,
                ...eventPayload,
            });
        });
    },

    resolve(id: string, result: boolean) {
        const cb = listeners.get(id);
        if (cb) cb(result);
        listeners.delete(id);
        contents.delete(id);
    },

    // shortcuts
    info(message: string, title = "Info",
        text?: {
            confirmText?: string;
            cancelText?: string;
        },
        content?: ReactNode
    ) {
        return dialog.open({ type: "info", message, title, text, content });
    },

    success(message: string, title = "Success",
        text?: {
            confirmText?: string;
            cancelText?: string;
        },
        content?: ReactNode
    ) {
        return dialog.open({ type: "success", message, title, text, content });
    },

    error(message: string, title = "Error",
        text?: {
            confirmText?: string;
            cancelText?: string;
        },
        content?: ReactNode
    ) {
        return dialog.open({ type: "error", message, title, text, content });
    },

    warning(message: string, title = "Warning",
        text?: {
            confirmText?: string;
            cancelText?: string;
        },
        content?: ReactNode
    ) {
        return dialog.open({ type: "warning", message, title, text, content });
    },

    question(message: string, title = "Confirm",
        text?: {
            confirmText?: string;
            cancelText?: string;
        },
        content?: ReactNode
    ) {
        return dialog.open({ type: "question", message, title, text, content });
    },
};