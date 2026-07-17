import { useCallback, useEffect, useRef, useState } from "react";
import { RotateCw } from "lucide-react";

import AppWindow, { hideWindow } from "../components/AppWindow";
import Button from "../components/Button";
import Input from "../components/Input";

import { tctService } from "../api/services/tct.service";
import { getDelayRequest, useAppStore } from "../stores/app.store";
import Switch from "../components/Switch";
import { dialog } from "../service/dialog.service";
import { invoke } from "@tauri-apps/api/core";
import { useLocation } from "react-router-dom";
import { Loading } from "../components/Loading";

interface IProgs {
    params: Record<string, any>
}
export default function LoginTctPage({ params }: IProgs) {
    const location = useLocation();
    // const loading = useLoading.getState();
    const login = useAppStore((s) => s.login);
    const savePassword = useAppStore((s) => s.savePassword);
    const setLogin = useAppStore((s) => s.setLogin);

    const [password, setPassword] = useState(
        savePassword?.[params.username] ?? ""
    );

    const [captchaImage, setCaptchaImage] = useState("");
    const [loadingCaptcha, setLoadingCaptcha] = useState(false);
    const [remember, setRemember] = useState(true);
    const [ckey, setCkey] = useState("");
    const [cvalue, setCvalue] = useState("");

    const getInvoiceTCT = useCallback(async (token: string) => {
        await hideWindow();
        if (!params.type) return;
        const delay = getDelayRequest();
        await invoke("start_invoice_tct_sync", {
            invoiceType: params.type,
            fromDate: params.fromDate,
            toDate: params.toDate,
            token: token,
            delay: delay
        });
    }, [params]);

    const loadCaptcha = useCallback(async () => {
        setLoadingCaptcha(true);
        const res = await tctService.getCaptcha();
        if (res) {
            setCaptchaImage(res.captcha);
            setCkey(res.key);
        }

        setLoadingCaptcha(false);
    }, []);
    const reConnect = useRef(params.reConnect);

    useEffect(() => {
        if (
            login &&
            login.username === params.username && !reConnect.current
        ) {
            invoke("set_current_route", {
                route: location.pathname,
            });
            getInvoiceTCT(login.token);
            return;
        }
        invoke("page_ready", { name: 'loginTct' });
        loadCaptcha();
    }, [login, loadCaptcha, getInvoiceTCT, location.key]);

    const [loading, setLoading] = useState(false);
    const handleLogin = async () => {
        if (!password || !cvalue) {
            await dialog.warning(`Bạn phải nhập ${!password ? 'mật khẩu' : 'captcha'}!`);
            return;
        }
        // loading.show("...")
        setLoading(true);
        const res = await tctService.login({
            username: params.username,
            password,
            ckey,
            cvalue,
        });
        // loading.hide();
        setLoading(false);
        if (!res) return;
        reConnect.current = false;
        setLogin({
            source: "TCT",
            taxCode: params.taxCode,
            username: params.username,
            password: remember ? password : "",
            token: res.token,
            idAccount: "",
            reConnect: true
        });
        // await getInvoiceTCT(res.token);
    };

    if (
        login &&
        login.username === params.username && !reConnect.current
    ) {
        return null;
    }

    return (
        <AppWindow title="Đăng nhập" icon="User">
            <div className="flex h-full flex-col gap-4 p-6 w-95">

                <div className="mb-2 w-full flex text-center justify-center">
                    <img
                        src="/tct.png"
                        alt="splash"
                        className="w-lg h-15 object-contain"
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
                <div className="space-y-2">

                    <label className="text-sm font-medium">
                        Mã xác nhận
                    </label>

                    <div className="flex items-center gap-2">

                        <img
                            src={captchaImage}
                            alt="captcha"
                            className="
                                h-10
                                flex-1
                                rounded-lg
                                border
                                object-contain
                            "
                        />

                        <Button
                            iconOnly
                            variant="secondary"
                            loading={loadingCaptcha}
                            icon={<RotateCw size={16} />}
                            onClick={loadCaptcha}
                        />

                    </div>

                </div>

                <Input
                    placeholder="Nhập mã captcha"
                    value={cvalue}
                    onChange={(e) =>
                        setCvalue(e.target.value)
                    }
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