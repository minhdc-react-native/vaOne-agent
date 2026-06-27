import { useCallback, useLayoutEffect, useState } from "react";
import { RotateCw, Eye, EyeOff } from "lucide-react";
import { useLocation } from "react-router-dom";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { tctService } from "../api/services/tct.service";
import { sendNotification } from "@tauri-apps/plugin-notification";
import { useAppStore } from "../stores/app.store";
import { trayApi } from "../api/axios/axiosClient";

export default function LoginPage() {
    const token = useAppStore(store => store.tokenTCT);
    const location = useLocation();
    const params = location.state;

    const [username] = useState(params.username);
    const [password, setPassword] = useState(params.password);
    const [cvalue, setCvalue] = useState("");
    const [ckey, setCkey] = useState("");
    const [captchaImage, setCaptchaImage] = useState("");
    const [loadingCaptcha, setLoadingCaptcha] = useState(false);
    const [showPassword, setShowPassword] = useState(false);
    const setTokenTCT = useAppStore(appStore => appStore.setTokenTCT);
    const getInvoiceTCT = useCallback(async () => {
        await getCurrentWindow().hide();
        const payload = {
            route: "/get-invoice-tct",
            data: {
                taxCode: username,
                type: params.type,
                fromDate: params.fromDate,
                toDate: params.toDate,
                screen: {
                    title: "Lấy hóa đơn",
                    width: 400,
                    height: 510
                }
            }
        }
        await trayApi.post("/open_tray_page", payload);
    }, [username, params])
    const loadCaptcha = useCallback(async () => {
        if (token !== null) {
            await getInvoiceTCT();
            return;
        }
        setLoadingCaptcha(true);
        const res = await tctService.getCaptcha();
        if (res) {
            setCaptchaImage(res.captcha);
            setCkey(res.key);
        } else {
            sendNotification({
                title: "vaOne Agent",
                body: "Không thể lấy mã Captcha!",
            });
        }
        setLoadingCaptcha(false);
    }, [token]);

    useLayoutEffect(() => {
        loadCaptcha();
    }, [location.state]);

    const handleLogin = async () => {
        const res = await tctService.login({
            username, password, ckey, cvalue
        });
        if (res) {
            setTokenTCT(res.token);
            await getInvoiceTCT();
        }
    };

    return (
        <div className="login-page">
            <div className="login-logo">
                <h2>vaOne Agent</h2>
                <span>Đăng nhập Hóa đơn điện tử</span>
            </div>

            <div className="field">
                <label>Tên đăng nhập</label>

                <input
                    className="app-input"
                    value={username}
                    readOnly
                />
            </div>

            <div className="field">
                <label>Mật khẩu</label>

                <div className="password-box">
                    <input
                        className="app-input"
                        type={showPassword ? "text" : "password"}
                        value={password}
                        autoFocus
                        onChange={(e) => setPassword(e.target.value)}
                    />

                    <button
                        type="button"
                        className="eye-btn"
                        onClick={() => setShowPassword((v) => !v)}
                    >
                        {showPassword ? (
                            <EyeOff size={18} />
                        ) : (
                            <Eye size={18} />
                        )}
                    </button>
                </div>
            </div>

            <div className="field">
                <label>Mã xác nhận</label>

                <div className="captcha-box">
                    <img
                        src={captchaImage}
                        alt="Captcha"
                    />

                    <button
                        type="button"
                        className="app-button refresh"
                        onClick={loadCaptcha}
                        disabled={loadingCaptcha}
                    >
                        <RotateCw
                            size={18}
                            className={loadingCaptcha ? "spin" : ""}
                        />
                    </button>
                </div>
            </div>

            <div className="field">
                <input
                    className="app-input"
                    placeholder="Nhập mã captcha"
                    value={cvalue}
                    onChange={(e) => setCvalue(e.target.value)}
                />
            </div>

            <button
                className="app-button login-btn"
                onClick={handleLogin}
            >
                Đăng nhập
            </button>
        </div>
    );
}