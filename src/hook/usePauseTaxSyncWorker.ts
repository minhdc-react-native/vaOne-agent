
import { useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { showNotification } from "../lib/notification";

export function usePauseTaxSyncWorker() {
    const stop = useCallback(async () => {
        await invoke("stop_tax_sync_worker").catch((error) => {
            showNotification({
                type: "warning",
                message: "Không thể dừng tax sync worker:"
            })
        });
    }, [])
    const start = useCallback(async () => {
        await invoke("start_tax_sync_worker").catch((error) => {
            showNotification({
                type: "warning",
                message: "Không thể khởi động lại tax sync worker:"
            })
        });
    }, [])
    return {
        stop,
        start
    }
}
