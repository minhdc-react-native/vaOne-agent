import { useLayoutEffect, useState } from "react";
import { generatePdf, printPdf } from "../api/pdf";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import AppWindow from "../components/AppWindow";
import Button from "../components/Button";
import { Printer } from "lucide-react";
import { useLocation } from "react-router-dom";
const template = {
    page: {
        width: 210,
        height: 297,
    },
    elements: [
        {
            "id": "stglgae01",
            "name": "text_stgl",
            "type": "text",
            "x": 57.4818141050897,
            "y": 40.71982281284607,
            "width": 504.2753783684016,
            "height": 20,
            "content": "<b>Địa chỉ:</b> {value}",
            "fieldName": "storeInfo.address",
            "childElements": [],
            "style": {
                "backgroundColor": "transparent",
                "opacity": 1,
                "fontSize": 11,
                "color": "#000000",
                "textAlign": "left",
                "borderRadius": 4,
                "padding": 8
            }
        }
    ],
};
const data = {
    "storeInfo": {
        "address": "123 Nguyễn Trãi, Thanh Xuân, Hà Nội"
    }
};

export const PreviewReport = () => {
    const location = useLocation();
    const { report, data: data_report } = location.state || {};
    const [url, setUrl] = useState({ url: "", path: "" });
    const handleGenerate = async () => {
        await invoke("page_ready", { name: 'report' });
        const path = await generatePdf(report || template, data_report || data) as string;
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