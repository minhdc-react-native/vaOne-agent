import { type ReactNode } from "react";
import { X } from "lucide-react";
import Button from "./Button";
import { trayApi } from "../api/axios/axiosClient";
import { useEscape } from "../hook/useEscape";

interface AppWindowProps {
    title: string;
    children: ReactNode;
    content?: ReactNode;
    disableClose?: boolean
}
export const hideWindow = async () => {
    await trayApi.post("/open_tray_page", {
        route: "/blank",
        data: {},
    });
};

export default function AppWindow({
    title,
    children,
    content,
    disableClose = false
}: AppWindowProps) {

    useEscape(hideWindow, !disableClose);

    return (
        <div className="flex h-screen flex-col bg-white">
            {/* Title Bar */}
            <header
                data-tauri-drag-region
                className="
                    flex
                    h-10
                    items-center
                    justify-between
                    border-b
                    border-gray-200
                    px-3
                    select-none
                    shrink-0
                "
            >
                <span
                    data-tauri-drag-region
                    className="flex-1 text-sm font-semibold text-gray-800"
                >
                    {title}
                </span>
                {content}
                <Button
                    disabled={disableClose}
                    onClick={hideWindow} variant="ghost"
                    iconOnly icon={<X size={15} />}
                />
            </header>

            {/* Content */}
            <main className="flex-1 overflow-auto">
                {children}
            </main>
        </div>
    );
}