import { useLayoutEffect, useState } from "react";
import { generatePdf, printPdf } from "../api/pdf";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import AppWindow from "../components/AppWindow";
import { useLocation } from "react-router-dom";
import { imageUrlToBase64 } from "../api/services/image.service";

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
    useLayoutEffect(() => {
        handleGenerate();
    }, [])

    const onPrint = async () => {
        await printPdf(url.path);
    }
    return (
        <AppWindow title={"Preview Report"}
        // content={
        //     (
        //         <div className="flex gap-1 py-2 pr-2 mr-2 border-r border-gray-300">
        //             <Button
        //                 variant="ghost"
        //                 onClick={onPrint}
        //                 icon={<Printer size={16} />}
        //                 className="h-7!"
        //             >
        //                 In
        //             </Button>
        //         </div>
        //     )
        // }
        >
            {url.url && <iframe
                src={url.url}
                className="w-full h-full"
            />}
        </AppWindow>
    )
}