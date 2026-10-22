import { useEffect } from "react";
import AppWindow from "../components/AppWindow";
import { invoke } from "@tauri-apps/api/core";
import TaxAccountManager from "../features/TaxAccountManager";
import { Pause } from "lucide-react";

export const TaxConfig = () => {
    useEffect(() => {
        invoke("page_ready", { name: "taxConfig" });
    }, []);

    return (
        <AppWindow title={"Quản lý kết nối, đồng bộ dữ liệu với tổng cục thuế"} icon="Unplug"
        // content={
        //     <div className="flex gap-1 items-center mr-5 px-2 py-1 bg-amber-50 border border-amber-300 rounded-full">
        //         <Pause className="text-red-500" size={15} />
        //         <span className="text-xs">dịch vụ đồng bộ tạm dừng cho đến khi bạn đóng màn hình</span>
        //     </div>
        // }
        >
            <TaxAccountManager />
        </AppWindow>
    )
}