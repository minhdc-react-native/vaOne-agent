import { useEffect, useState } from "react";
import AppWindow from "../components/AppWindow";
import { invoke } from "@tauri-apps/api/core";
import TaxAccountManager from "../features/TaxAccountManager";
import { usePauseTaxSyncWorker } from "../hook/usePauseTaxSyncWorker";
import { ArrowBigDownDash, Cable, Loader, SquarePause, SquarePlay } from "lucide-react";
import Button from "../components/Button";
import { Tooltip } from "../components/Tooltip/Tooltip";
import { useLatest } from "../hook/useLatest";
import { dialog } from "../service/dialog.service";
import ModalOverlay from "../features/ModalOverlay";
import { useAppStore } from "../stores/app.store";
import { listen } from "@tauri-apps/api/event";
import { Loading } from "../components/Loading";
export const KEY_TAX_ACCOUNT_SCROLL_TOP = 'tax-account-scroll-top'
export const TaxConfig = () => {
    const { loading,
        isRunning,
        setWorkerRunning
    } = usePauseTaxSyncWorker();

    useEffect(() => {
        invoke("page_ready", { name: "taxConfig" });

        return () => {
            sessionStorage.removeItem(KEY_TAX_ACCOUNT_SCROLL_TOP);
        }
    }, []);
    const openOverlay = useAppStore(appStore => appStore.openOverlay);
    const setOpenOverlay = useAppStore(appStore => appStore.setOpenOverlay);
    const onProgressTask = async () => {
        try {
            setOpenOverlay(true);
            await invoke("open_modal_window", { route: "/progress-task", name: "taxConfig" });
        } catch (error) {
            console.error("Không thể mở tiến trình đồng bộ:", error);
        }
    };

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
                <div className="mr-4 flex gap-2 items-center">
                    {isRunning && <div className="relative">
                        <Loading className="h-6 w-6" />
                        <div className="absolute top-0 left-0">
                            <Tooltip title={"Tiến trình đồng bộ"}><Button variant="ghost" iconOnly={true} className="border-transparent!" onClick={onProgressTask}
                                icon={<ArrowBigDownDash className="text-orange-500" size={15} />}
                            /></Tooltip>
                        </div>
                    </div>}

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
            <ModalOverlay open={openOverlay} />
            <TaxAccountManager isRunning={isRunning} />
        </AppWindow>
    )
}