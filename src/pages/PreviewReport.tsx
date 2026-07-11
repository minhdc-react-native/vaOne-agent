import { useEffect, useLayoutEffect, useState } from "react";
import { generatePdf, printPdf } from "../api/pdf";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import AppWindow from "../components/AppWindow";
import { useLocation } from "react-router-dom";
import { imageUrlToBase64 } from "../api/services/image.service";
import { listen } from "@tauri-apps/api/event";
import Progress from "../components/Progress";

export interface IPdfProgress {
    message: string;
    currentReport: number,
    totalReport: number,
    current: number;
    total: number;
}

export const PreviewReport = () => {
    const location = useLocation();
    const { reports, datas } = location.state || {};
    const [url, setUrl] = useState({ url: "", path: "" });
    const handleGenerate = async () => {
        await invoke("page_ready", { name: 'report' });
        for (const report of reports) {
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
        }

        const path = await generatePdf(reports, datas) as string;
        const url = convertFileSrc(path);
        setUrl({ url, path });
    };

    const [progress, setProgress] = useState<IPdfProgress>({
        message: "Đang thực hiện...",
        currentReport: 0,
        totalReport: 0,
        current: 0,
        total: 0
    });

    useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setup = async () => {
            unlisten = await listen("pdf-progress", (event: any) => {
                setProgress(prev => ({ ...prev, ...event.payload }));
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
        <AppWindow title={"Preview Report"} icon="Printer"
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
                <div className="w-100 flex flex-col gap-2 justify-center">
                    <div className="text-lg font-semibold">{`Báo cáo: ${progress.currentReport}/${progress.totalReport}`}</div>
                    <Progress
                        message={progress.message}
                        value={progress.current}
                        total={progress.total}
                    />
                </div>
            </div>}

        </AppWindow>
    )
}