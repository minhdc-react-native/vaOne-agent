import { useEffect } from "react";
import { checkUpdate } from "../service/updater";
import { trayApi } from "../api/axios/axiosClient";

const CHECK_INTERVAL = 5 * 60 * 1000; // 5 phút

export function useUpdater() {
    useEffect(() => {
        let checking = false;

        const run = async () => {
            if (checking) return;

            checking = true;

            try {
                const update = await checkUpdate();
                if (update) {
                    await trayApi.post("/open_tray_page", {
                        route: "/update",
                        data: {},
                    });
                }
            } catch (err) {
                console.error("Check update failed:", err);
            } finally {
                checking = false;
            }
        };

        // Kiểm tra ngay khi khởi động
        run();

        // Sau đó mỗi 5 phút
        const timer = setInterval(run, CHECK_INTERVAL);

        return () => clearInterval(timer);
    }, []);
}