
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { showNotification } from "../lib/notification";

export function usePauseTaxSyncWorker() {
    const [isRunning, setIsRunning] = useState(false);
    const [loading, setLoading] = useState(true);
    const checkWorkerStatus = async () => {
        const running = await invoke<boolean>(
            "is_tax_sync_worker_running",
        );
        setIsRunning(running);
        setLoading(false);
    };

    useEffect(() => {
        checkWorkerStatus();
    }, [])

    const stopSyncWorker = useCallback(async () => {
        await invoke("stop_tax_sync_worker").catch((error) => {
            showNotification({
                type: "warning",
                message: "Không thể dừng tax sync worker:"
            })
        });
    }, [])

    const startSyncWorker = useCallback(async () => {
        await invoke("start_tax_sync_worker").catch((error) => {
            showNotification({
                type: "warning",
                message: "Không thể khởi động lại tax sync worker:"
            })
        });
    }, [])

    const setWorkerRunning = useCallback(async (running: boolean) => {
        setLoading(true);
        try {
            if (running) {
                await startSyncWorker();
            } else {
                await stopSyncWorker();
            }
            setIsRunning(running);
        } catch (error) {

        } finally {
            setLoading(false);
        }
    }, []);


    return {
        loading,
        isRunning,
        setWorkerRunning,
        stopSyncWorker,
        startSyncWorker
    }
}
