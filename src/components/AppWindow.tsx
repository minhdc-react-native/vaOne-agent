import { useEffect, type ReactNode } from "react";
import { Maximize, Minimize, X } from "lucide-react";
import Button from "./Button";
import { trayApi } from "../api/axios/axiosClient";
import { useEscape } from "../hook/useEscape";
import { useLocation } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { IconLucide, IconName } from "./IconLucide";
import { Tooltip } from "./Tooltip/Tooltip";

interface AppWindowProps {
    icon?: IconName,
    title?: string;
    children: ReactNode;
    content?: ReactNode;
    disableClose?: boolean;
    callBack?: {
        onConfirm?: () => Promise<boolean>;
        onClose?: () => void;
    }
    fullScreen?: boolean;
    setFullScreen?: (fullScreen: boolean) => void;
}
export const hideWindow = async (callBack?: {
    onConfirm?: () => Promise<boolean>;
    onClose?: () => void;
}) => {
    if (callBack?.onConfirm) {
        const confirmed = await callBack.onConfirm();

        if (!confirmed) return;
    }

    await trayApi.post("/open_tray_page", {
        route: "/blank",
        data: {},
    });

    callBack?.onClose?.();
};

export default function AppWindow({
    icon,
    title,
    children,
    content,
    disableClose = false,
    callBack,
    fullScreen,
    setFullScreen
}: AppWindowProps) {
    const location = useLocation();
    useEscape(hideWindow, !disableClose);

    useEffect(() => {
        const path = location.pathname.split("/").slice(0, 2).join("/");
        invoke("set_current_route", {
            route: path,
        });
    }, [location.pathname]);

    return (
        <div className="flex h-screen flex-col bg-white">
            {/* Title Bar */}
            <header
                className="
                    flex
                    gap-2
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
                {icon && <IconLucide
                    name={icon}
                />}
                {title && <span
                    data-tauri-drag-region
                    className="cursor-pointer flex-1 text-sm font-semibold text-gray-800"
                >
                    {title}
                </span>}
                {content}
                {fullScreen !== undefined && <Tooltip title={fullScreen ? "Thu nhỏ" : "Toàn màn hình"}><Button
                    onClick={() => setFullScreen?.(!fullScreen)} variant="ghost"
                    iconOnly icon={fullScreen ? <Minimize size={15} /> : <Maximize size={15} />}
                /></Tooltip>}
                {!disableClose && <Tooltip title="Thoát"><Button
                    // disabled={disableClose}
                    onClick={() => {
                        if (fullScreen) setFullScreen?.(false);
                        hideWindow(callBack);
                    }} variant="ghost"
                    iconOnly icon={<X size={15} />}
                /></Tooltip>}
            </header>

            {/* Content */}
            <main className="flex flex-1 overflow-auto">
                {children}
            </main>
        </div>
    );
}