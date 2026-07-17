import { invoke } from "@tauri-apps/api/core";
import { trayApi } from "../api/axios/axiosClient";
import Button from "../components/Button";
import { useEffect } from "react";
import { Divider } from "../components/Divider";
import { useEscape } from "../hook/useEscape";
import AppWindow, { hideWindow } from "../components/AppWindow";

export default function QuitPage() {
    const quit = async () => {
        await invoke("quit_app");
    };

    useEffect(() => {
        invoke("page_ready", { name: "quit" });
    }, []);

    return (
        <AppWindow title="Thoát ứng dụng" icon="LogOut">
            <div className="w-87.5 h-full p-4">
                <p className="flex justify-center">
                    Bạn có muốn thoát
                </p>
                <p className="flex justify-center items-center">
                    <span className="font-semibold text-red-700 pr-2 text-2xl">vaOne</span><span className="text-gray-500 pr-2 text-2xl">plugin</span>không?
                </p>
                <div className="mt-6 flex justify-end gap-2">
                    <Button
                        variant="secondary"
                        onClick={hideWindow}
                    >
                        Hủy
                    </Button>

                    <Button
                        onClick={quit}
                    >
                        Thoát
                    </Button>
                </div>
            </div>
        </AppWindow>
    );
}