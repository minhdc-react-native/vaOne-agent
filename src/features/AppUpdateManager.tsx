import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { IUpdateAvailable, useAppUpdate } from "../hook/useAppUpdate";

export function AppUpdateManager() {
    const [update, setUpdate] =
        useState<IUpdateAvailable | null>(null);

    const [installing, setInstalling] = useState(false);

    const handleUpdateAvailable = useCallback(
        (data: IUpdateAvailable) => {
            setUpdate(data);
        },
        []
    );

    useAppUpdate({
        onUpdateAvailable: handleUpdateAvailable,
    });

    const handleInstall = async () => {
        try {
            setInstalling(true);

            await invoke("install_update");
        } catch (error) {
            console.error("[UPDATE]", error);
            setInstalling(false);
        }
    };

    if (!update) return null;

    return (
        <div className="modal">
            <div className="modal-content">
                <h3>Cập nhật VAOne Agent</h3>

                <p>
                    Phiên bản hiện tại:
                    {update.currentVersion}
                </p>

                <p>
                    Phiên bản mới:
                    {update.newVersion}
                </p>

                <div className="release-notes">
                    {update.notes}
                </div>

                <button
                    disabled={installing}
                    onClick={() => setUpdate(null)}
                >
                    Để sau
                </button>

                <button
                    disabled={installing}
                    onClick={handleInstall}
                >
                    {installing
                        ? "Đang cập nhật..."
                        : "Cập nhật ngay"}
                </button>
            </div>
        </div>
    );
}