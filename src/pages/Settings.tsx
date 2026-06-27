import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
    enable,
    disable,
    isEnabled,
} from "@tauri-apps/plugin-autostart";
import { useAppStore } from "../stores/app.store";
import { Line } from "../components/Line";
// import { sendNotification } from "@tauri-apps/plugin-notification";

interface AgentInfo {
    name: string;
    version: string;
    os: string;
}

export default function SettingsPage() {
    const [info, setInfo] = useState<AgentInfo>();
    const delayRequest = useAppStore(store => store.delayRequest);
    const setDelayRequest = useAppStore(store => store.setDelayRequest)
    const [delay, setDelay] = useState(0);
    const [autoStart, setAutoStart] = useState(false);
    const [savedAutoStart, setSavedAutoStart] = useState(false);

    const loadData = async () => {
        try {
            const enabled = await isEnabled();
            setAutoStart(enabled);
            setSavedAutoStart(enabled);
            setDelay(delayRequest);
            const agent = await invoke<AgentInfo>("get_agent_info");
            setInfo(agent);
        } catch (e) {
            console.error(e);
        }
    };

    useEffect(() => {
        loadData();
        const reload = () => loadData();
        window.addEventListener("settings-open", reload);
        return () => {
            window.removeEventListener("settings-open", reload);
        };
    }, [delayRequest]);

    const save = async () => {
        try {
            if (autoStart !== savedAutoStart) {
                if (autoStart) {
                    await enable();
                } else {
                    await disable();
                }

                setSavedAutoStart(autoStart);
            }
            if (delay !== delayRequest) setDelayRequest(delay);
            await getCurrentWindow().hide();
        } catch (e) {
            console.error(e);
        }
    };

    const close = async () => {
        setAutoStart(savedAutoStart);
        await getCurrentWindow().hide();
    };

    if (!info) {
        return <div className="settings">Loading...</div>;
    }

    return (
        <div className="settings">
            <div className="content">
                <div className="row">
                    <span>Version</span>
                    <strong>{info.version}</strong>
                </div>

                <div className="row">
                    <span>OS</span>
                    <strong>{info.os}</strong>
                </div>

                <div className="row">
                    <span>Auto Start</span>

                    <label className="switch">
                        <input
                            type="checkbox"
                            checked={autoStart}
                            onChange={(e) =>
                                setAutoStart(e.target.checked)
                            }
                        />
                        <span className="slider" />
                    </label>
                </div>
                <div className="row">
                    <span>Delay request (ms)</span>
                    <input
                        className="app-input"
                        style={{ width: 100 }}
                        value={delay}
                        type="number"
                        onChange={(e) =>
                            setDelay(Number(e.target.value))
                        }
                    />
                </div>
            </div>
            <div className="footer">
                <button
                    className="app-button"
                    onClick={save}
                >
                    Save
                </button>

                <button
                    className="app-button app-button-outline"
                    onClick={close}
                >
                    Close
                </button>
            </div>
        </div>
    );
}