import { useCallback, useEffect, useState } from "react";
import Progress from "../components/Progress";
import { formatDate, tctService } from "../api/services/tct.service";
import { sendNotification } from "@tauri-apps/plugin-notification";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { message } from "@tauri-apps/plugin-dialog";
import { useLocation } from "react-router-dom";
import { Line } from "../components/Line";
const TYPE_LOADING = {
    1: "Hóa đơn mua vào",
    2: "Hóa đơn bán ra",
    3: "Hóa đơn mua vào MTT",
    4: "Hóa đơn bán ra MTT"
}
export const GetInvoiceTct = () => {
    const location = useLocation();
    const { taxCode, type, fromDate, toDate } = location.state;
    const [info, setInfo] = useState({ current: 0, total: 0 });
    const [invoices, setInvoices] = useState<any[]>([]);
    // const [invoicesWidthDetail, setInvoicesWidthDetail] = useState<any[]>([]);
    const [currentInvoice, setCurrentInvoice] = useState<any>({});
    const getInvoice = useCallback(async () => {
        const dataAll = await tctService.getInvoice(type, { fromDate, toDate });
        setInfo(prev => ({ ...prev, total: dataAll.length }));
        setInvoices(dataAll);
    }, []);

    const { tdlap, khhdon, shdon } = currentInvoice;

    const getInvoiceDetail = useCallback(async (datas: any[]) => {
        const dataAllWidthDetail = await tctService.getInvoiceDetail(datas,
            (numInvoice, currentInvoice) => {
                setInfo(prev => ({ ...prev, current: numInvoice }));
                setCurrentInvoice(currentInvoice);
            }
        );
        console.log('dataAllWidthDetail>>', dataAllWidthDetail);
        // setInvoicesWidthDetail(dataAllWidthDetail);
        // await getCurrentWindow().hide();
        await message(`Đã lấy được ${dataAllWidthDetail.length}/${datas.length} hóa đơn !`, {
            title: "Thông báo",
            kind: "info", // "info" | "warning" | "error"
        });
    }, []);

    useEffect(() => {
        getInvoice();
    }, []);

    useEffect(() => {
        if (invoices.length === 0) return;
        getInvoiceDetail(invoices);
    }, [invoices]);

    return (
        <div style={{
            padding: 10
        }}>
            <div className="invoice-loading">
                <span className="value">
                    Mã số thuế: <strong>{taxCode}</strong>
                </span>
                <span className="value">{`Loại: ${TYPE_LOADING[type as 1 | 2 | 3 | 4]}`}</span>
                <span className="value">{`Từ ngày: ${formatDate(fromDate)} đến ngày: ${formatDate(toDate)}`}</span>
                <Line />
                <span className="value">
                    Đang tải hóa đơn: Ký hiệu: {khhdon}, Ngày:{" "}
                    <strong>{formatDate(tdlap)}</strong>, Số hóa đơn:{" "}
                    <strong style={{ color: "red" }}>{shdon}</strong>
                </span>
                {invoices.length === 0 && <span className="value"><strong style={{ color: "red" }}>Không có số hóa đơn nào!</strong></span>}
            </div>
            <Progress
                value={info.current}
                total={info.total}
            />
            <div style={{
                height: invoices.length === 0 ? 250 : 280,
                border: "1px solid darkred",
                borderRadius: 8,
                marginTop: 10,
                backgroundColor: "#ffffff",
            }}>

            </div>
        </div>
    );
}