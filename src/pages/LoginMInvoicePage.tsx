import { invoke } from "@tauri-apps/api/core";
import AppWindow from "../components/AppWindow";
import Button from "../components/Button";
import { useEffect } from "react";
interface IProgs {
    params: Record<string, any>
}
export default function LoginMInvoicePage({ params }: IProgs) {
    useEffect(() => {
        invoke("page_ready", { name: 'loginMInvoice' });
    }, []);
    return (
        <AppWindow title="Đăng nhập" icon="User">
            <div className="flex h-full flex-col gap-4 p-4 w-95">

                <div className="mb-2 text-center">
                    <h2 className="text-xl font-bold">
                        M-Invoice
                    </h2>

                    <p className="text-sm text-gray-500">
                        Đăng nhập Hóa đơn điện tử
                    </p>
                </div>

                <Button
                    className="mt-2 w-full"
                    onClick={() => { }}
                >
                    Đăng nhập
                </Button>
            </div>
        </AppWindow>
    );
}