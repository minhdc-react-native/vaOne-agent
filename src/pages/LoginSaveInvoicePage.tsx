import { invoke } from "@tauri-apps/api/core";
import AppWindow, { hideWindow } from "../components/AppWindow";
import Button from "../components/Button";
import { useCallback, useEffect, useRef, useState } from "react";
import Input from "../components/Input";
import { getDelayRequest, useAppStore } from "../stores/app.store";
import Switch from "../components/Switch";
import { dialog } from "../service/dialog.service";
import { useLoading } from "../service/loading.service";
import { saveInvoiceService } from "../api/services/saveInvoice.service";
import { useLocation } from "react-router-dom";
import { Loading } from "../components/Loading";
interface IProgs {
    params: Record<string, any>
}
export default function LoginSaveInvoicePage({ params }: IProgs) {
    const location = useLocation();
    // const loading = useLoading.getState();
    const [remember, setRemember] = useState(true);
    const login = useAppStore(s => s.login);
    const setLogin = useAppStore(s => s.setLogin);
    const savePassword = useAppStore((s) => s.savePassword);

    const [username, setUserName] = useState(params.username);

    const [password, setPassword] = useState(
        savePassword?.[params.username] ?? ""
    );

    const getInvoice = useCallback(async (token: string, idAccount: string) => {
        await hideWindow();
        if (!params.type) return;
        const delay = getDelayRequest();
        await invoke("start_save_invoice_sync", {
            invoiceType: params.type,
            fromDate: params.fromDate,
            toDate: params.toDate,
            token,
            delay: 100,
            idAccount
        });
    }, [params]);

    const reConnect = useRef(params.reConnect);
    useEffect(() => {
        if (
            login &&
            login.taxCode === params.taxCode && !reConnect.current
        ) {
            invoke("set_current_route", {
                route: location.pathname,
            });
            getInvoice(login.token, login.idAccount!);
            return;
        }
        invoke("page_ready", { name: 'loginSaveInvoice' });
    }, [login, location.key]);

    const [loading, setLoading] = useState(false);
    const handleLogin = async () => {
        if (!password) {
            await dialog.warning(`Bạn phải nhập ${!password ? 'mật khẩu' : 'captcha'}!`);
            return;
        }
        setLoading(true);
        // loading.show("...")
        const res = await saveInvoiceService.apiToken({
            taxCode: params.taxCode,
            username: params.username,
            password,
        });
        setLoading(false);
        // loading.hide();
        if (!res) return;
        reConnect.current = false;
        setLogin({
            source: "SAVE-INVOICE",
            taxCode: params.taxCode,
            username: params.username,
            password: remember ? password : "",
            token: res.token,
            idAccount: res.id,
            reConnect: true
        });
    };

    if (
        login &&
        login.taxCode === params.taxCode && !reConnect.current
    ) {
        return null;
    }

    return (
        <AppWindow title="Đăng nhập" icon="User">
            <div className="flex h-full flex-col gap-2 p-6 w-95">

                <div className="mb-2 w-full flex text-center justify-center">
                    <img
                        src="/save-invoice.png"
                        alt="splash"
                        className="w-lg h-40 object-contain"
                    />
                </div>
                <div className="text-sm flex justify-center bg-amber-50 border border-gray-300 rounded-2xl p-2">Mã số thuế:<span className="pl-2 font-semibold">{params.taxCode}</span></div>

                <Input
                    label="Tên đăng nhập"
                    value={username}
                    onChange={(e) =>
                        setUserName(e.target.value)
                    }
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
                {loading ? <div className="flex justify-center"><Loading /></div> : <Button
                    className="mt-2 w-full"
                    onClick={handleLogin}
                >
                    Đăng nhập
                </Button>}
            </div>
        </AppWindow>
    );
}