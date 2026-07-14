import { invoke } from "@tauri-apps/api/core";
import AppWindow from "../components/AppWindow";
import Button from "../components/Button";
import { useEffect, useState } from "react";
import Input from "../components/Input";
import { useAppStore } from "../stores/app.store";
import Switch from "../components/Switch";
interface IProgs {
    params: Record<string, any>
}
export default function LoginMInvoicePage({ params }: IProgs) {
    const [remember, setRemember] = useState(true);
    const savePasswordLoginMInvoice = useAppStore((s) => s.savePasswordLoginMInvoice);
    const [password, setPassword] = useState(
        savePasswordLoginMInvoice?.[params.username] ?? ""
    );

    useEffect(() => {
        invoke("page_ready", { name: 'loginMInvoice' });
    }, []);

    return (
        <AppWindow title="Đăng nhập" icon="User">
            <div className="flex h-full flex-col gap-2 p-4 w-95">

                <div className="mb-2 w-full flex text-center justify-center">
                    <img
                        src="/m-invoice.png"
                        alt="splash"
                        className="w-lg h-40 object-contain"
                    />
                </div>
                <Input
                    label="Tên đăng nhập"
                    value={params.username}
                    readOnly
                />

                <Input
                    label="Mật khẩu"
                    password
                    autoFocus
                    value={password}
                    onChange={(e) =>
                        setPassword(e.target.value)
                    }
                />
                <Switch
                    label="Lưu mật khẩu"
                    checked={remember}
                    onChange={setRemember}
                />
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