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
interface IProgs {
    params: Record<string, any>
}
export default function LoginSaveInvoicePage({ params }: IProgs) {
    const location = useLocation();
    const loading = useLoading.getState();
    const [remember, setRemember] = useState(true);
    const tokenSaveInvoice = useAppStore(s => s.tokenSaveInvoice);
    const setLoginSaveInvoice = useAppStore(s => s.setLoginSaveInvoice);
    const savePasswordLoginSaveInvoice = useAppStore((s) => s.savePasswordLoginSaveInvoice);

    const [username, setUserName] = useState(params.username);

    const [password, setPassword] = useState(
        savePasswordLoginSaveInvoice?.[params.username] ?? ""
    );

    const getInvoice = useCallback(async (token: string, idAccount: string) => {
        await hideWindow();
        const delay = getDelayRequest();
        await invoke("start_save_invoice_sync", {
            invoiceType: params.type,
            fromDate: params.fromDate,
            toDate: params.toDate,
            token,
            delay,
            idAccount
        });
    }, [params]);
    useEffect(() => {
        if (
            tokenSaveInvoice &&
            tokenSaveInvoice.taxCode === params.taxCode
        ) {
            getInvoice(tokenSaveInvoice.token, tokenSaveInvoice.idAccount!);
            return;
        }
        invoke("page_ready", { name: 'loginSaveInvoice' });
    }, [tokenSaveInvoice, params.taxCode, location.key]);

    const handleLogin = async () => {
        if (!password) {
            await dialog.warning(`Bạn phải nhập ${!password ? 'mật khẩu' : 'captcha'}!`);
            return;
        }
        loading.show("...")
        const res = await saveInvoiceService.apiToken({
            taxCode: params.taxCode,
            username: params.username,
            password
        });
        loading.hide();
        if (!res) return;

        setLoginSaveInvoice({
            taxCode: params.taxCode,
            username: params.username,
            password: remember ? password : "",
            token: res.token,
            idAccount: res.id
        });
        // await getInvoiceTCT(res.token);
    };

    if (
        tokenSaveInvoice &&
        tokenSaveInvoice.username === params.username
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