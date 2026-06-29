import { useEffect, type ReactNode } from "react";
import { getCurrentWindow, PhysicalSize } from "@tauri-apps/api/window";
import { X } from "lucide-react";
import Button from "./Button";
import Tooltip from "./Tooltip";
import { trayApi } from "../api/axios/axiosClient";

interface AppWindowProps {
    title: string;
    children: ReactNode;
    disableClose?: boolean
}

export default function AppWindow({
    title,
    children,
    disableClose = false
}: AppWindowProps) {

    const hideWindow = async () => {
        await trayApi.post("/open_tray_page", {
            route: "/blank",
            data: {
                screen: {
                    width: 10,
                    height: 10,
                }
            },
        });
    };

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
                    className="text-sm font-semibold text-gray-800"
                >
                    {title}
                </span>
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