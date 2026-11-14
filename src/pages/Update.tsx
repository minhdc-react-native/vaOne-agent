import { useEffect, useMemo, useState } from "react";
import AppWindow, { hideWindow, IWindowConfig } from "../components/AppWindow";
import { listen } from "@tauri-apps/api/event";
import Progress from "../components/Progress";

export interface IProgress {
    message: string;
    "currentVersion": string,
    "newVersion": string,
    downloaded: number;
    total: number;
    finish: boolean;
}

export const UpdatePage = () => {
    const [progress, setProgress] = useState<IProgress>({
        message: "Đang thực hiện...",
        currentVersion: "",
        newVersion: "",
        downloaded: 0,
        total: 0,
        finish: false
    });
    useEffect(() => {
        if (progress.finish) hideWindow();
    }, [progress.finish]);

    useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setup = async () => {
            unlisten = await listen("update-progress", (event: any) => {
                setProgress(prev => ({ ...prev, ...event.payload }));
            });
        };

        setup();

        return () => {
            unlisten?.();
        };
    }, []);

    const config = useMemo<IWindowConfig>(() => {
        return { name: "update" }
    }, []);

    return (
        <AppWindow title="Cập nhật" icon="Download" disableClose={true} config={config}>
            <div className="w-82.5 flex flex-col gap-2 p-4 justify-center">
                <span className="text-sm text-gray-500">
                    Tải file cài đặt...
                </span>
                <Progress
                    message={progress.message}
                    value={progress.downloaded}
                    total={progress.total}
                    unit="MB"
                />
            </div>
        </AppWindow>
    )
}