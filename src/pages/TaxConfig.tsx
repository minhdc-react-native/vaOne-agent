import { useEffect } from "react";
import AppWindow from "../components/AppWindow";
import { invoke } from "@tauri-apps/api/core";
import TaxAccountManager from "../features/TaxAccountManager";
import { usePauseTaxSyncWorker } from "../hook/usePauseTaxSyncWorker";
import { Loader, SquarePause, SquarePlay } from "lucide-react";
import Button from "../components/Button";
import { Tooltip } from "../components/Tooltip/Tooltip";

export const TaxConfig = () => {
    const { loading,
        isRunning,
        setWorkerRunning
    } = usePauseTaxSyncWorker();

    useEffect(() => {
        invoke("page_ready", { name: "taxConfig" });
    }, []);

    return (
        <AppWindow title={"Quản lý kết nối, đồng bộ dữ liệu với tổng cục thuế"} icon="Unplug"
            content={
                <div className="mr-4">
                    {loading ? <Loader className="animate-spin" size={15} /> :
                        <div className="flex gap-1 items-center">
                            <span className="text-sm font-medium">{"Dịch vụ đồng bộ"}</span>
                            <Tooltip title={isRunning ? "Bấm để dừng dịch vụ" : "Bấm để kích hoạt chạy dịch vụ"}><Button variant="ghost" iconOnly={true} className="border-transparent!" onClick={() => setWorkerRunning(!isRunning)}
                                icon={isRunning ? <SquarePause className="text-purple-500" /> : <SquarePlay className="text-blue-500" />}
                            /></Tooltip>
                        </div>}
                </div>
            }
        >
            <TaxAccountManager isRunning={isRunning} />
        </AppWindow>
    )
}