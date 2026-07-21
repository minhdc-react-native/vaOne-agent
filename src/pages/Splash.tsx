import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "react";

export const SplashScreen = () => {
    useEffect(() => {
        invoke("page_ready", { name: "splash", show: false });
    }, []);
    return (<div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-white">
        {/* Logo */}
        <img
            src="/LogoVacom.png"
            alt="splash"
            className="w-34 h-11 object-contain"
        />
    </div>);
}