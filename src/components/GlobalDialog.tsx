import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import Dialog from "./Dialog";
import { dialog } from "../service/dialog.service";

export type DialogPayload = {
    id: string;
    type: any;
    title?: string;
    message?: string;
    text?: {
        confirmText?: string;
        cancelText?: string;
    }
};

export function GlobalDialog() {
    const [current, setCurrent] = useState<DialogPayload | null>(null);

    useEffect(() => {
        console.log("🔔 Dialog listener mounted");

        const unlistenPromise = listen("dialog:open", (event: any) => {
            console.log("📩 received dialog:", event.payload);
            setCurrent(event.payload);
        });

        return () => {
            unlistenPromise.then((f) => f());
        };
    }, []);

    if (!current) return null;

    return (
        <Dialog
            open={true}
            type={current.type}
            title={current.title}
            message={current.message}
            confirmText={current.text?.confirmText ?? "Đồng ý"}
            cancelText={current.text?.cancelText ?? "Hủy bỏ"}
            onCancel={() => {
                dialog.resolve(current.id, false);
                setCurrent(null);
            }}
            onConfirm={() => {
                dialog.resolve(current.id, true);
                setCurrent(null);
            }}
        />
    );
}