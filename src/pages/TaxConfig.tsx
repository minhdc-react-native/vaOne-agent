import { useEffect } from "react";
import AppWindow from "../components/AppWindow";
import { invoke } from "@tauri-apps/api/core";
import TaxAccountManager from "../features/TaxAccountManager";

export const TaxConfig = () => {
    useEffect(() => {
        invoke("page_ready", { name: "taxConfig" });
    }, []);

    return (
        <AppWindow title={"Quản lý kết nối với tổng cục thuế"} icon="Unplug">
            <TaxAccountManager />
        </AppWindow>
    )
}