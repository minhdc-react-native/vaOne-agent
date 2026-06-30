import { invoke } from "@tauri-apps/api/core";
import { trayApi } from "../api/axios/axiosClient";
import Button from "../components/Button";
import { useEffect } from "react";
import { Divider } from "../components/Divider";
import { useEscape } from "../hook/useEscape";

export default function QuitPage() {
    const quit = async () => {
        await invoke("quit_app");
    };

    const cancel = async () => {
        await trayApi.post("/open_tray_page", {
            route: "/blank",
            data: {},
        });
    };

    useEffect(() => {
        invoke("page_ready", { name: "quit" });
    }, []);

    useEscape(cancel, true);

    return (
        <div className="flex h-screen items-center justify-center">
            <div className="w-full h-full p-4">
                <h2 className="text-xl font-bold">
                    Thoát ứng dụng
                </h2>

                <p className="my-4">
                    Bạn có muốn thoát vaOne plugin không?
                </p>
                <Divider />
                <div className="mt-6 flex justify-end gap-2">
                    <Button
                        variant="secondary"
                        onClick={cancel}
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
        </div>
    );
}