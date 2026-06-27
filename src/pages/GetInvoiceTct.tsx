import { useCallback, useEffect, useState } from "react";
import Progress from "../components/Progress";
import { formatDate, tctService } from "../api/services/tct.service";
import { sendNotification } from "@tauri-apps/plugin-notification";
export const GetInvoiceTct = () => {
    const [info, setInfo] = useState({ current: 0, total: 0 });
    const [invoices, setInvoices] = useState<any[]>([]);
    // const [invoicesWidthDetail, setInvoicesWidthDetail] = useState<any[]>([]);
    const [currentInvoice, setCurrentInvoice] = useState<any>({});
    const getInvoice = useCallback(async () => {
        const dataAll = await tctService.getInvoice(2, { fromDate: '2026-05-28', toDate: '2026-06-27' });
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
        sendNotification({
            title: "Nhận dữ liệu",
            body: "Quá trình nhận dữ liệu đã hoàn tất!",
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
            padding: 20
        }}>
            <div className="invoice-loading">
                <div className="row">
                    <span className="label">Ngày</span>
                    <span className="value">{formatDate(tdlap) || "--"}</span>
                </div>
                <div className="row">
                    <span className="label">Ký hiệu</span>
                    <span className="value">{khhdon || "--"}</span>
                </div>
                <div className="row">
                    <span className="label">Số hóa đơn</span>
                    <span className="value">{shdon || "--"}</span>
                </div>
            </div>
            <Progress
                value={info.current}
                total={info.total}
            />
            <div style={{
                height: 200,
                border: "1px solid darkred",
                borderRadius: 8,
                marginTop: 10,
                backgroundColor: "#ffffff",
            }}>

            </div>
        </div>
    );
}