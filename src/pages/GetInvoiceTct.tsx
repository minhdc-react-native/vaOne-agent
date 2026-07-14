import { useCallback, useEffect, useMemo, useState } from "react";
import { useLocation } from "react-router-dom";

import Progress from "../components/Progress";
import { formatDate, tctService } from "../api/services/tct.service";
import { Divider } from "../components/Divider";
import { dialog } from "../service/dialog.service";
import AppWindow from "../components/AppWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { trayApi } from "../api/axios/axiosClient";
import { useAppStore } from "../stores/app.store";
import { useLoading } from "../service/loading.service";
import { Loading } from "../components/Loading";
import { useCancellation } from "../api/useCancellation";
import { invoke } from "@tauri-apps/api/core";
import Button from "../components/Button";

const TYPE_LOADING: Record<number, string> = {
    1: "Hóa đơn mua vào",
    2: "Hóa đơn bán ra",
    3: "Hóa đơn mua vào MTT",
    4: "Hóa đơn bán ra MTT",
};

type LocationState = {
    source: number;
    taxCode: string;
    type: 1 | 2 | 3 | 4;
    fromDate: string;
    toDate: string;
};

type Invoice = any;

export const GetInvoiceTct = () => {
    const location = useLocation();
    const { isCancelled } = useCancellation();
    const params = location.state as LocationState;
    const { taxCode, type, fromDate, toDate } = params;
    const setLoginTct = useAppStore((s) => s.setLoginTct);
    const [disableClose, setDisableClose] = useState(false);
    const [progress, setProgress] = useState({ current: 0, total: 0 });
    const [invoices, setInvoices] = useState<Invoice[]>([]);
    const [currentInvoice, setCurrentInvoice] = useState<Invoice | null>(null);
    const titleType = useMemo(
        () => TYPE_LOADING[type],
        [type]
    );
    const loading = useLoading.getState();
    const loadInvoices = useCallback(async () => {
        setInvoices([]);
        loading.show("...");
        try {
            const data = await tctService.getInvoice(type, {
                fromDate,
                toDate,
            }, isCancelled);
            if (isCancelled()) return;
            setInvoices(data);
            setProgress((p) => ({ ...p, total: data.length }));
        } catch (err) {
            if (isCancelled()) return;
            console.error("getInvoice error:", err);
            await dialog.error("Không thể tải danh sách hóa đơn");
        } finally {
            loading.hide();
        }
    }, [type, fromDate, toDate]);

    useEffect(() => {
        invoke("page_ready", { name: 'getInvoiceTct' });
    }, []);

    const loadInvoiceDetails = useCallback(
        async (data: Invoice[]) => {
            setDisableClose(true);
            try {
                const result = await tctService.getInvoiceDetail(
                    data,
                    (index, current, isError) => {
                        if (isCancelled()) return;
                        if (!isError) {
                            setProgress((p) => ({ ...p, current: index }));
                            setCurrentInvoice(current);
                        }
                    },
                    isCancelled
                );
                // await getCurrentWindow().hide();
                if (isCancelled()) return;
                setDisableClose(false);
                await dialog.success(`Đã lấy được ${result.length}/${data.length} hóa đơn!`);
            } catch (err) {
                console.error("getInvoiceDetail error:", err);
                if (isCancelled()) return;
                setDisableClose(false);
                await dialog.error("Lỗi khi tải chi tiết hóa đơn");
            } finally {

            }
        },
        []
    );

    useEffect(() => {
        loadInvoices();
    }, [loadInvoices]);

    useEffect(() => {
        if (invoices.length > 0) {
            loadInvoiceDetails(invoices);
        }
    }, [invoices, loadInvoiceDetails]);

    const { khhdon, shdon, tdlap } = currentInvoice || {};

    const reLogin = useCallback(async () => {
        setLoginTct(null);
        await getCurrentWindow().hide();
        await trayApi.post("/open_tray_page", {
            route: "/login",
            data: {
                ...params,
                username: params.taxCode
            }
        });
    }, [params]);

    return (
        <AppWindow title="Tải hóa đơn..." disableClose={disableClose} icon="File">
            <div className="space-y-2 p-6 w-100">
                {/* Info panel */}
                <div className="space-y-1 text-sm">
                    <div className="flex gap-1 items-center">
                        <p className="flex-1">
                            Mã số thuế: <strong>{taxCode}</strong>
                        </p>
                        {disableClose && <Loading />}
                    </div>
                    <p>Loại: {titleType}</p>
                    <p>
                        Từ ngày: {formatDate(fromDate)} → {formatDate(toDate)}
                    </p>
                    <Divider />
                    <p>
                        Đang tải: Ký hiệu: {khhdon ?? "--"}, Ngày:{" "}
                        <strong>{tdlap ? formatDate(tdlap) : "--"}</strong>, Số hóa đơn:{" "}
                        <strong className="text-red-500">{shdon ?? "--"}</strong>
                    </p>

                    {invoices.length === 0 && (
                        <p className="text-red-500 font-semibold">
                            Không có số hóa đơn nào!
                        </p>
                    )}
                </div>

                {/* Progress */}
                <Progress value={progress.current} total={progress.total} />

                {/* Content box */}
                <div
                    className="border border-red-800 rounded-md bg-white"
                    style={{
                        height: invoices.length === 0 ? 250 : 280,
                    }}
                />
                {/* <div className="flex justify-end">
                    <Button
                        className="mt-2 w-40"
                        onClick={reLogin} disabled={disableClose}
                    >
                        Đăng nhập lại
                    </Button>
                </div> */}
            </div>
        </AppWindow>
    );
};