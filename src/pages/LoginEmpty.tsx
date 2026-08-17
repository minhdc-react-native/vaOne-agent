import { useEffect } from "react";
import AppWindow from "../components/AppWindow";
import { invoke } from "@tauri-apps/api/core";

export default function LoginEmpty() {
    useEffect(() => {
        invoke("page_ready", { name: 'loginEmpty' });
    }, [])
    return (
        <AppWindow title="Đăng nhập" icon="User">
            <div className="flex h-full flex-col gap-2 p-4 w-95">
                <div className="text-sm flex flex-col justify-center bg-amber-50 border border-gray-300 rounded-2xl p-2">
                    <span className="font-semibold mb-1">Bạn chưa đăng nhập!</span>
                    <span>Hãy đăng nhập chính thức từ trang web</span>
                </div>
            </div>
        </AppWindow>
    );
}