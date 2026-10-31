import { useEffect, useState } from "react";
import { emit, listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import AppWindow from "../components/AppWindow";
import { Loader } from "lucide-react";
import { Loading } from "../components/Loading";
import { ProgressTaskItem } from "../features/ProgressTaskItem";
import { useAppStore } from "../stores/app.store";
import { showNotification } from "../lib/notification";
import { dialog } from "../service/dialog.service";

export type InvoiceTask =
    | "download"
    | "complete_details"
    | "upload";

export type TaskStatus =
    | "running"
    | "completed"
    | "completed_with_errors"
    | "failed"
    | "cancelled";

export interface TaskProgress {
    tax_code: string;
    task: InvoiceTask;
    status: TaskStatus;
    completed: number;
    total: number | null;
    message: string;

    invoice_type: number;

    invoice_date?: string | null;
    invoice_number?: number | null;
    invoice_serial?: string | null;

    success?: boolean | null;
    error?: string | null;
    current_invoice?: unknown | null;
}

interface ProgressTaskProps {
    className?: string;
}


export default function ProgressTask({
    className = "",
}: ProgressTaskProps) {
    const [tasks, setTasks] = useState<TaskProgress[]>([]);

    useEffect(() => {
        let mounted = true;
        let unlisten: (() => void) | undefined;

        const updateTask = (progress: TaskProgress) => {
            if (!mounted) return;

            const key = `${progress.tax_code}:${progress.task}`;

            setTasks((prev) => {
                if (progress.status !== "running") {
                    return prev.filter(
                        (item) => `${item.tax_code}:${item.task}` !== key
                    );
                }

                const index = prev.findIndex(
                    (item) => `${item.tax_code}:${item.task}` === key
                );

                if (index === -1) {
                    return [...prev, progress];
                }

                const next = [...prev];
                next[index] = progress;

                return next;
            });
        };

        const init = async () => {
            try {
                // Đăng ký listener trước để tránh bỏ lỡ event.
                const stop = await listen<TaskProgress>(
                    "tax-sync-progress",
                    (event) => {
                        updateTask(event.payload);
                    }
                );

                if (!mounted) {
                    stop();
                    return;
                }

                unlisten = stop;

                // Lấy lại các tác vụ đang chạy khi mở UI.
                const current = await invoke<TaskProgress[]>(
                    "sync_current_progress"
                );

                if (mounted) {
                    setTasks((prev) => {
                        // Giữ lại các event mới hơn đã nhận trong lúc invoke.
                        const map = new Map<string, TaskProgress>();

                        current.forEach((item) => {
                            map.set(`${item.tax_code}:${item.task}`, item);
                        });

                        prev.forEach((item) => {
                            map.set(`${item.tax_code}:${item.task}`, item);
                        });

                        return Array.from(map.values());
                    });
                }
            } catch (error) {
                console.error("[ProgressTask] Init error:", error);
            }
        };

        init();

        return () => {
            mounted = false;
            unlisten?.();
        };
    }, []);

    useEffect(() => {
        invoke("page_ready", { name: "progressTask" });
    }, []);

    const beforeClose = async () => {
        await emit('modal_closed');
    }

    return (
        <AppWindow title={"Theo dõi tiến trình đồng bộ dữ liệu"} icon="Loader" callBack={{ beforeClose }}>
            <div className={`space-y-3 w-full m-4 ${className}`}>
                {tasks.length === 0 && <NoTask />}
                {tasks.map((task) => <ProgressTaskItem key={`${task.tax_code}:${task.task}`} task={task} />)}
            </div>
        </AppWindow>
    );
}

const NoTask = () => {
    return (
        <div className="flex min-h-80 flex-col items-center justify-center rounded-lg border border-dashed border-gray-300 bg-gray-50/70 px-6 text-center">
            <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-gray-100">
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="30"
                    height="30"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.5"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    className="text-gray-400"
                >
                    <path d="M20 7v5h-5" />
                    <path d="M4 17v-5h5" />
                    <path d="M5.6 9a7 7 0 0 1 11.5-2L20 12" />
                    <path d="M18.4 15a7 7 0 0 1-11.5 2L4 12" />
                </svg>
            </div>

            <h3 className="text-sm font-semibold text-gray-700">
                Không có tiến trình đang chạy
            </h3>

            <p className="mt-1 max-w-75 text-xs leading-5 text-gray-500">
                Hiện tại không có tác vụ đồng bộ hóa đơn nào đang được thực hiện.
                Các tiến trình mới sẽ tự động hiển thị tại đây.
            </p>

            <div className="mt-4 flex items-center gap-2 text-xs text-gray-400">
                <span className="h-1.5 w-1.5 rounded-full bg-gray-300" />
                Hệ thống đang tạm dừng xử lý
            </div>
        </div>
    );
}