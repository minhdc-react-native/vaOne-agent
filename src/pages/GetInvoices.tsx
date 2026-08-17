import { useEffect, useMemo } from "react";

import Progress from "../components/Progress";
import { Divider } from "../components/Divider";
import AppWindow from "../components/AppWindow";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../stores/app.store";
import { formatDate } from "../api/services/tct.service";

const TYPE_LOADING: Record<number, string> = {
    1: "Hóa đơn mua vào",
    2: "Hóa đơn bán ra",
    3: "Hóa đơn mua vào MTT",
    4: "Hóa đơn bán ra MTT",
};

export const GetInvoices = () => {
    const login = useAppStore(store => store.login);
    const syncProgress = useAppStore(store => store.syncProgress)
    const titleType = useMemo(
        () => TYPE_LOADING[login?.info.invoiceType],
        [login?.info.invoiceType]
    );
    useEffect(() => {
        invoke("page_ready", { name: 'getInvoiceTct' });
    }, []);

    return (
        <AppWindow title="Tải hóa đơn..." icon="Download">
            <div className="space-y-2 p-6 w-100">
                {/* Info panel */}
                <div className="space-y-1 text-sm">
                    <div className="flex gap-1 items-center">
                        <p className="flex-1">
                            Mã số thuế: <strong>{login?.taxCode}</strong>
                        </p>
                    </div>
                    <p>Loại: {titleType}</p>
                    <p>
                        Từ ngày: {formatDate(login?.info.fromDate)} → {formatDate(login?.info.toDate)}
                    </p>
                    <Divider />
                    <p>
                        Đang tải: Ký hiệu: {syncProgress?.invoice?.invoiceSerial}
                    </p>
                    <p>
                        Ngày:{" "}
                        <strong>{formatDate(syncProgress?.invoice?.invoiceDate)}</strong>, Số hóa đơn: {" "}
                        <strong className="text-red-500">{syncProgress?.invoice?.invoiceNumber}</strong>
                    </p>

                </div>

                {/* Progress */}
                <Progress value={syncProgress?.completed || 0} total={syncProgress?.total || 0} />

                {/* Content box */}
                <div
                    className="border border-red-800 rounded-md bg-white"
                    style={{
                        height: 280
                    }}
                />
            </div>
        </AppWindow>
    );
};