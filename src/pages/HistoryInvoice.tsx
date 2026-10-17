import { useCallback, useEffect, useMemo } from "react";
import AppWindow from "../components/AppWindow";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate, useParams } from "react-router-dom";
import { useTaxAccounts } from "../hook/useTaxAccounts";

export const HistoryInvoice = () => {
    const {
        accounts,
    } = useTaxAccounts();

    const { taxCode } = useParams<{ taxCode: string }>();

    const account = useMemo(() => accounts.find(acc => acc.tax_code === taxCode), [taxCode]);

    const navigate = useNavigate();
    useEffect(() => {
        invoke("page_ready", { name: "historyInvoice", show: true });
    }, []);

    const onClose = useCallback(() => {
        navigate("/tax-config");
    }, [])

    return (
        <AppWindow title={"Lịch sử truyền nhận hóa đơn"} icon="History"
            onClose={onClose}
        >
            <div className="w-full h-full border flex items-center">
                <div className="p-4 overflow-auto">{JSON.stringify(account)}</div>
            </div>
        </AppWindow>
    )
}