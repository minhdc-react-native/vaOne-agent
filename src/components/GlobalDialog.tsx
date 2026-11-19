import { ReactNode, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import Dialog, { DialogSize, DialogType } from "./Dialog";
import { contents, dialog, listeners } from "../service/dialog.service";

export type DialogPayload = {
    id: string;
    type: DialogType;
    title?: string;
    message?: string;
    text?: {
        confirmText?: string;
        cancelText?: string;
    },
    content?: ReactNode,
    closeIcon?: boolean,
    size?: DialogSize
};

export function GlobalDialog() {
    const [current, setCurrent] = useState<DialogPayload | null>(null);

    useEffect(() => {
        let unlisten: (() => void) | undefined;
        let disposed = false;

        listen("dialog:open", (event: any) => {
            setCurrent(event.payload);
        }).then((fn) => {
            if (disposed) {
                fn();
            } else {
                unlisten = fn;
            }
        });

        return () => {
            disposed = true;
            unlisten?.();
        };
    }, []);

    function closeDialog(id: string, result: boolean) {
        const resolve = listeners.get(id);
        listeners.delete(id);
        resolve?.(result);
        setCurrent(null);
    }

    if (!current) return null;

    const content = contents.get(current.id);
    return (
        <Dialog
            open={true}
            type={current.type}
            title={current.title}
            message={current.message}
            confirmText={current.text?.confirmText ?? "Đồng ý"}
            cancelText={current.text?.cancelText ?? "Hủy bỏ"}
            onCancel={() => {
                closeDialog(current.id, false);
            }}
            onConfirm={() => {
                closeDialog(current.id, true);
            }}
            content={content}
            size={current.size}
            closeIcon={current.closeIcon}
        />
    );
}