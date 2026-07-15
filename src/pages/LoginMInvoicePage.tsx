import { invoke } from "@tauri-apps/api/core";
import AppWindow, { hideWindow } from "../components/AppWindow";
import Button from "../components/Button";
import { useCallback, useEffect, useState } from "react";
import Input from "../components/Input";
import { getDelayRequest, useAppStore } from "../stores/app.store";
import Switch from "../components/Switch";
import { useLocation } from "react-router-dom";
import { useLoading } from "../service/loading.service";
import { dialog } from "../service/dialog.service";
import { mInvoiceService } from "../api/services/mInvoice.service";
interface IProgs {
    params: Record<string, any>
}
export default function LoginMInvoicePage({ params }: IProgs) {
    const location = useLocation();
    const loading = useLoading.getState();
    const [remember, setRemember] = useState(true);
    const tokenMInvoice = useAppStore(s => s.tokenMInvoice);
    const setLoginMInvoice = useAppStore(s => s.setLoginMInvoice);
    const savePasswordLoginMInvoice = useAppStore((s) => s.savePasswordLoginMInvoice);

    const [username, setUserName] = useState(params.username);

    const [password, setPassword] = useState(
        savePasswordLoginMInvoice?.[params.username] ?? ""
    );

    const getInvoice = useCallback(async (token: string, taxCode: string) => {
        await hideWindow();
        const delay = getDelayRequest();
        await invoke("start_m_invoice_sync", {
            invoiceType: params.type,
            fromDate: params.fromDate,
            toDate: params.toDate,
            token,
            delay,
            taxCode
        });
    }, [params]);
    useEffect(() => {
        if (
            tokenMInvoice &&
            tokenMInvoice.taxCode === params.taxCode
        ) {
            getInvoice(tokenMInvoice.token, tokenMInvoice.taxCode!);
            return;
        }
        invoke("page_ready", { name: 'loginSaveInvoice' });
    }, [tokenMInvoice, params.taxCode, location.key]);

    const handleLogin = async () => {
        if (!password) {
            await dialog.warning(`Bạn phải nhập ${!password ? 'mật khẩu' : 'captcha'}!`);
            return;
        }
        loading.show("...")
        const res = await mInvoiceService.apiToken({
            taxCode: params.taxCode,
            username: params.username,
            password,
        });
        loading.hide();
        if (!res) return;

        setLoginMInvoice({
            taxCode: params.taxCode,
            username: params.username,
            password: remember ? password : "",
            token: res.token,
            idAccount: res.id
        });
    };

    if (
        tokenMInvoice &&
        tokenMInvoice.taxCode === params.taxCode
    ) {
        return null;
    }

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
                    label="Mã số thuế"
                    value={params.taxCode}
                    readOnly
                />

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
                <Button
                    className="mt-2 w-full"
                    onClick={handleLogin}
                >
                    Đăng nhập
                </Button>
            </div>
        </AppWindow>
    );
}