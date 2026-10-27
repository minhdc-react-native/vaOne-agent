import { useEffect } from "react";
import AppWindow from "../components/AppWindow";
import { invoke } from "@tauri-apps/api/core";
import TaxAccountManager from "../features/TaxAccountManager";
import { usePauseTaxSyncWorker } from "../hook/usePauseTaxSyncWorker";
import { Loader, SquarePause, SquarePlay } from "lucide-react";
import Button from "../components/Button";
import { Tooltip } from "../components/Tooltip/Tooltip";
import { useLatest } from "../hook/useLatest";
import { dialog } from "../service/dialog.service";
export const KEY_TAX_ACCOUNT_SCROLL_TOP = 'tax-account-scroll-top'
export const TaxConfig = () => {
    const { loading,
        isRunning,
        setWorkerRunning
    } = usePauseTaxSyncWorker();

    const refIsRunning = useLatest(isRunning);

    useEffect(() => {
        invoke("page_ready", { name: "taxConfig" });

        return () => {
            sessionStorage.removeItem(KEY_TAX_ACCOUNT_SCROLL_TOP);
        }
    }, []);

    const onConfirm = async () => {
        const confirmed = await dialog.question(
            `Bạn đang dừng dịch vụ, bạn nên bật lại dịch vụ đồng bộ!, bạn vẫn muốn thoát màn hình? `,
            "Thoát màn hình",
            {
                confirmText: "Có",
                cancelText: "Không"
            }
        );
        return confirmed;
    }
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
            callBack={!isRunning ? { onConfirm } : undefined}
        >
            <TaxAccountManager isRunning={isRunning} />
        </AppWindow>
    )
}