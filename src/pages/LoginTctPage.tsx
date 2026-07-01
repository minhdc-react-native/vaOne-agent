import { useCallback, useEffect, useLayoutEffect, useState } from "react";
import { RotateCw } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

import AppWindow from "../components/AppWindow";
import Button from "../components/Button";
import Input from "../components/Input";

import { formatDate, tctService } from "../api/services/tct.service";
import { trayApi } from "../api/axios/axiosClient";
import { getDelayRequest, useAppStore } from "../stores/app.store";
import Switch from "../components/Switch";
import { useLoading } from "../service/loading.service";
import { dialog } from "../service/dialog.service";
import { invoke } from "@tauri-apps/api/core";
const getUrl = (type: 1 | 2 | 3 | 4, fromDate: string, toDate: string) => {
    /*
        1: Mua vào
        2: Bán ra
        3: Mua vào MTT
        4: Bán ra MTT
    */
    const sizePage = 50;
    let url = `https://hoadondientu.gdt.gov.vn/api/query/invoices/sold?sort=tdlap:desc&size=${sizePage}&search=tdlap=ge=${formatDate(fromDate)}T00:00:00;tdlap=le=${formatDate(toDate)}T23:59:59`;
    switch (type) {
        case 1:
        case 3:
            url = `https://hoadondientu.gdt.gov.vn/api/query/invoices/purchase?sort=tdlap:desc&size=${sizePage}&search=tdlap=ge=${formatDate(fromDate)}T00:00:00;tdlap=le=${formatDate(toDate)}T23:59:59;ttxly==5`;
            break;
        case 2:
        case 4:
            url = `https://hoadondientu.gdt.gov.vn/api/query/invoices/sold?sort=tdlap:desc&size=${sizePage}&search=tdlap=ge=${formatDate(fromDate)}T00:00:00;tdlap=le=${formatDate(toDate)}T23:59:59`;
            break;
    }
    return url;
};


interface IProgs {
    params: Record<string, any>
}
export default function LoginTctPage({ params }: IProgs) {
    const loading = useLoading.getState();
    const tokenTct = useAppStore((s) => s.tokenTct);
    const savePasswordLoginTct = useAppStore((s) => s.savePasswordLoginTct);
    const setLoginTct = useAppStore((s) => s.setLoginTct);

    const [password, setPassword] = useState(
        savePasswordLoginTct?.[params.username] ?? ""
    );

    const [captchaImage, setCaptchaImage] = useState("");
    const [loadingCaptcha, setLoadingCaptcha] = useState(false);
    const [remember, setRemember] = useState(true);
    const [ckey, setCkey] = useState("");
    const [cvalue, setCvalue] = useState("");

    const getInvoiceTCT = useCallback(async (token: string) => {
        await getCurrentWindow().hide();
        const delay = getDelayRequest();
        await invoke("start_invoice_tct_sync", {
            url: getUrl(params.type, params.fromDate, params.toDate),
            token, delay
        });
        // await trayApi.post("/open_tray_page", {
        //     route: "/get-invoice-tct",
        //     data: {
        //         taxCode: params.username,
        //         type: params.type,
        //         fromDate: params.fromDate,
        //         toDate: params.toDate
        //     },
        // });
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
    useEffect(() => {
        if (
            tokenTct &&
            tokenTct.username === params.username
        ) {
            getInvoiceTCT(tokenTct.token);
            return;
        }
        invoke("page_ready", { name: 'loginTct' });
        loadCaptcha();
    }, [tokenTct, params.username, loadCaptcha, getInvoiceTCT]);

    const handleLogin = async () => {
        if (!password || !cvalue) {
            await dialog.warning(`Bạn phải nhập ${!password ? 'mật khẩu' : 'captcha'}!`);
            return;
        }
        loading.show("...")
        const res = await tctService.login({
            username: params.username,
            password,
            ckey,
            cvalue,
        });
        loading.hide();
        if (!res) return;

        setLoginTct({
            username: params.username,
            password,
            token: res.token,
        });

        await getInvoiceTCT(res.token);
    };

    if (
        tokenTct &&
        tokenTct.username === params.username
    ) {
        return null;
    }

    return (
        <AppWindow title="Đăng nhập">
            <div className="flex h-full flex-col gap-4 p-4">

                <div className="mb-2 text-center">
                    <h2 className="text-xl font-bold">
                        vaOne Plugin
                    </h2>

                    <p className="text-sm text-gray-500">
                        Đăng nhập Hóa đơn điện tử
                    </p>
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