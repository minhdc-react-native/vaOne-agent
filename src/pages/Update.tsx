import { useEffect, useLayoutEffect, useState } from "react";
import { generatePdf, printPdf } from "../api/pdf";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import AppWindow, { hideWindow } from "../components/AppWindow";
import { useLocation } from "react-router-dom";
import { imageUrlToBase64 } from "../api/services/image.service";
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
    const handleGenerate = async () => {
        await invoke("page_ready", { name: 'update' });
    };

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
            await handleGenerate();
            unlisten = await listen("update-progress", (event: any) => {
                setProgress(prev => ({ ...prev, ...event.payload }));
            });
        };

        setup();

        return () => {
            unlisten?.();
        };
    }, []);

    return (
        <AppWindow title="Cập nhật" icon="Download" disableClose={true}>
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