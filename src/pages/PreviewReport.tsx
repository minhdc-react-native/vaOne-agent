import { useLayoutEffect, useState } from "react";
import { generatePdf, printPdf } from "../api/pdf";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import AppWindow from "../components/AppWindow";
import Button from "../components/Button";
import { Printer } from "lucide-react";
const template = {
    page: {
        width: 210,
        height: 297,
    },
    elements: [
        {
            type: "text",
            x: 20,
            y: 30,
            fontSize: 18,
            value: "ĐINH CÔNG MINH",
        },
        {
            type: "text",
            x: 20,
            y: 45,
            fontSize: 18,
            value: "Invoice",
        },
    ],
};

export const PreviewReport = () => {
    const [url, setUrl] = useState({ url: "", path: "" });
    const handleGenerate = async () => {
        invoke("page_ready", { name: 'report' });
        const path = await generatePdf(template) as string;
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
            content={
                (
                    <div className="flex gap-1 py-2 pr-2 mr-2 border-r border-gray-300">
                        <Button
                            variant="ghost"
                            onClick={onPrint}
                            icon={<Printer size={16} />}
                            className="h-7!"
                        >
                            In
                        </Button>
                    </div>
                )
            }
        >
            {url.url && <iframe
                src={url.url}
                className="w-full h-full"
            />}
        </AppWindow>
    )
}