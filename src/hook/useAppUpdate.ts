import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

export interface IUpdateAvailable {
    currentVersion: string;
    newVersion: string;
    notes: string;
}

interface IUseAppUpdate {
    onUpdateAvailable: (data: IUpdateAvailable) => void;
    onUpdateNotAvailable?: (version: string) => void;
    onUpdateError?: (message: string) => void;
}

export function useAppUpdate({
    onUpdateAvailable,
    onUpdateNotAvailable,
    onUpdateError,
}: IUseAppUpdate) {
    useEffect(() => {
        const unlisteners: Array<() => void> = [];
        let active = true;

        const setup = async () => {
            const unlistenUpdate = await listen<IUpdateAvailable>(
                "update-available",
                (event) => {
                    onUpdateAvailable(event.payload);
                }
            );

            const unlistenNotAvailable = await listen<string>(
                "update-not-available",
                (event) => {
                    onUpdateNotAvailable?.(event.payload);
                }
            );

            const unlistenError = await listen<string>(
                "update-check-failed",
                (event) => {
                    onUpdateError?.(event.payload);
                }
            );

            if (!active) {
                unlistenUpdate();
                unlistenNotAvailable();
                unlistenError();
                return;
            }

            unlisteners.push(
                unlistenUpdate,
                unlistenNotAvailable,
                unlistenError
            );

            await invoke("check_update", {
                silent: true,
            });

        };

        setup();

        return () => {
            active = false;
            unlisteners.forEach((unlisten) => unlisten());
        };
    }, [
        onUpdateAvailable,
        onUpdateNotAvailable,
        onUpdateError,
    ]);
}