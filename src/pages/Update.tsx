import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import AppWindow, { hideWindow } from "../components/AppWindow";
import Button from "../components/Button";
import { Divider } from "../components/Divider";
import { check, Update } from "@tauri-apps/plugin-updater";

export default function UpdatePage() {
    const [update, setUpdate] = useState<Update | null>(null);
    const checkUpdate = async () => {
        try {
            const update = await check();
            setUpdate(update);
        } catch (err) {
            console.error("Check update failed:", err);
        } finally {
        }
    };

    const handleUpdate = async () => {
        try {
            await hideWindow();
            await update!.downloadAndInstall();
        } catch (err) {
            console.error(err);
        }
    };

    useEffect(() => {
        invoke("page_ready", { name: "update" });
        checkUpdate();
    }, []);

    if (!update) {
        return (
            <AppWindow title="Nâng cấp" icon="Package">
                <div className="flex h-full w-82.5 items-center justify-center">
                    Loading...
                </div>
            </AppWindow>
        );
    }

    return (
        <AppWindow title="Nâng cấp" icon="Package">
            <div className="flex h-full w-82.5 flex-col justify-between p-4">

                <div className="space-y-5">

                    <div className="flex justify-between">
                        <span className="text-sm text-gray-500">
                            Phiên bản hiện tại
                        </span>

                        <span className="font-medium">
                            {update.currentVersion}
                        </span>
                    </div>

                    <div className="flex justify-between">
                        <span className="text-sm text-gray-500">
                            Phiên bản mới
                        </span>

                        <span className="font-medium text-green-600">
                            {update.version}
                        </span>
                    </div>

                    {update.date && (
                        <div className="flex justify-between">
                            <span className="text-sm text-gray-500">
                                Ngày phát hành
                            </span>

                            <span>
                                {new Date(update.date).toLocaleString("vi-VN")}
                            </span>
                        </div>
                    )}

                    <Divider />

                    <div>
                        <div className="mb-2 font-medium">
                            Nội dung cập nhật
                        </div>

                        <div className="max-h-48 overflow-auto rounded-md border bg-gray-50 p-3 text-sm whitespace-pre-wrap">
                            {update.body || "Không có mô tả."}
                        </div>
                    </div>

                </div>
                <Divider />
                <div className="flex justify-end gap-2">

                    <Button
                        variant="secondary"
                        onClick={hideWindow}
                    >
                        Hủy bỏ
                    </Button>

                    <Button onClick={handleUpdate}>
                        Cập nhật
                    </Button>

                </div>

            </div>
        </AppWindow>
    );
}