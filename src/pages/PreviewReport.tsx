import { useEffect, useLayoutEffect, useState } from "react";
import { generatePdf, printPdf } from "../api/pdf";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import AppWindow from "../components/AppWindow";
import { useLocation } from "react-router-dom";
import { imageUrlToBase64 } from "../api/services/image.service";
import { listen } from "@tauri-apps/api/event";
import Progress from "../components/Progress";

export enum PdfPhase {
    Preparing = "preparing",
    Paginating = "paginating",
    Rendering = "rendering",
    Saving = "saving",
    Completed = "completed",
}

const phaseText: Record<PdfPhase, string> = {
    [PdfPhase.Preparing]: "Đang chuẩn bị dữ liệu...",
    [PdfPhase.Paginating]: "Đang phân trang...",
    [PdfPhase.Rendering]: "Đang tạo PDF...",
    [PdfPhase.Saving]: "Đang lưu tệp...",
    [PdfPhase.Completed]: "Hoàn thành",
};


export interface IPdfProgress {
    phase: PdfPhase;
    current: number;
    total: number;
}

export const PreviewReport = () => {
    const location = useLocation();
    const { report, data: data_report } = location.state || {};
    const [url, setUrl] = useState({ url: "", path: "" });
    const handleGenerate = async () => {
        await invoke("page_ready", { name: 'report' });
        if (report.backgroundImage && report.backgroundImage.startsWith("http")) {
            report.backgroundImage = await imageUrlToBase64(report.backgroundImage);
        }
        for (const element of report.elements) {
            if (
                element.type === "image" &&
                typeof element.content === "string" &&
                element.content.startsWith("http")
            ) {
                element.content = await imageUrlToBase64(element.content);
            }
        }
        const path = await generatePdf(report, data_report) as string;
        const url = convertFileSrc(path);
        setUrl({ url, path });
    };

    const [progress, setProgress] = useState<IPdfProgress>({
        phase: PdfPhase.Preparing,
        current: 0,
        total: 0
    });

    useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setup = async () => {
            unlisten = await listen("pdf-progress", (event: any) => {
                setProgress(event.payload);
            });
            await handleGenerate();
        };

        setup();

        return () => {
            unlisten?.();
        };
    }, []);

    const onPrint = async () => {
        await printPdf(url.path);
    }
    useEffect(() => {
        console.log('progress>>', JSON.stringify(progress));
    }, [progress])
    return (
        <AppWindow title={"Preview Report"}
        // content={
        //     (
        //         <div className="flex gap-1 py-2 pr-2 mr-2 border-r border-gray-300 w-50">
        //             <Progress value={progress.current} total={progress.total} />
        //         </div>
        //     )
        // }
        >
            {url.url ? <iframe
                src={url.url}
                className="w-full h-full"
            /> : <div className="flex flex-1 items-center justify-center">
                <div className="w-100">
                    <Progress
                        value={progress.current}
                        total={progress.total}
                    />
                </div>
            </div>}
        </AppWindow>
    )
}