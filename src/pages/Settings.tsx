import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
    enable,
    disable,
    isEnabled,
} from "@tauri-apps/plugin-autostart";

import AppWindow, { hideWindow } from "../components/AppWindow";
import Button from "../components/Button";
import Input from "../components/Input";
import Switch from "../components/Switch";

import { useAppStore } from "../stores/app.store";
import { Divider } from "../components/Divider";
import { trayApi } from "../api/axios/axiosClient";

interface AgentInfo {
    name: string;
    version: string;
    os: string;
}

export default function SettingsPage() {
    const delayRequest = useAppStore((s) => s.delayRequest);
    const setDelayRequest = useAppStore((s) => s.setDelayRequest);

    const [info, setInfo] = useState<AgentInfo>();

    const [delay, setDelay] = useState(0);

    const [autoStart, setAutoStart] = useState(false);
    const [savedAutoStart, setSavedAutoStart] =
        useState(false);

    const loadData = async () => {
        try {
            const enabled = await isEnabled();

            setAutoStart(enabled);
            setSavedAutoStart(enabled);

            setDelay(delayRequest);

            const agent =
                await invoke<AgentInfo>("get_agent_info");

            setInfo(agent);
        } catch (err) {
            console.error(err);
        }
    };

    useEffect(() => {
        loadData();

        const reload = () => loadData();

        window.addEventListener(
            "settings-open",
            reload
        );

        return () => {
            window.removeEventListener(
                "settings-open",
                reload
            );
        };
    }, [delayRequest]);

    const handleSave = async () => {
        try {
            if (autoStart !== savedAutoStart) {
                if (autoStart) {
                    await enable();
                } else {
                    await disable();
                }

                setSavedAutoStart(autoStart);
            }

            if (delay !== delayRequest) {
                setDelayRequest(delay);
            }

            await hideWindow();
        } catch (err) {
            console.error(err);
        }
    };

    const handleClose = async () => {

        setAutoStart(savedAutoStart);
        setDelay(delayRequest);

        await hideWindow();
    };

    useEffect(() => {
        invoke("page_ready", { name: "settings" });
    }, []);

    if (!info) {
        return (
            <AppWindow title="Settings" icon="Cog">
                <div className="flex h-full w-82.5 items-center justify-center">
                    Loading...
                </div>
            </AppWindow>
        );
    }

    return (
        <AppWindow title="Settings" icon="Cog">
            <div className="flex h-full w-82.5 flex-col justify-between p-4">

                {/* Content */}

                <div className="space-y-5">

                    <div className="flex items-center justify-between">

                        <span className="text-sm text-gray-500">
                            Version
                        </span>

                        <span className="font-medium">
                            {info.version}
                        </span>

                    </div>

                    <div className="flex items-center justify-between">

                        <span className="text-sm text-gray-500">
                            Operating System
                        </span>

                        <span className="font-medium">
                            {info.os}
                        </span>

                    </div>

                    <Switch
                        label="Auto Start"
                        description="Khởi động cùng hệ điều hành."
                        checked={autoStart}
                        onChange={setAutoStart}
                    />

                    <Input
                        label="Delay Request (ms)"
                        labelPosition="left"
                        type="number"
                        value={delay}
                        onChange={(e) =>
                            setDelay(Number(e.target.value))
                        }
                    />

                </div>

                {/* Footer */}
                <Divider />
                <div className="flex justify-end gap-2">

                    <Button
                        variant="secondary"
                        onClick={handleClose}
                    >
                        Đóng
                    </Button>

                    <Button onClick={handleSave}>
                        Lưu
                    </Button>

                </div>

            </div>
        </AppWindow>
    );
}