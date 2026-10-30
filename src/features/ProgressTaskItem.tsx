import { Loading } from "../components/Loading";
import { InvoiceTask, TaskProgress, TaskStatus } from "../pages/ProgressTask";
const TASK_LABELS: Record<InvoiceTask, string> = {
    download: "Tải hóa đơn",
    complete_details: "Bổ sung thông tin",
    upload: "Đẩy dữ liệu lên ứng dụng",
};

interface IProgressTaskItem {
    task: TaskProgress
}
export const ProgressTaskItem = ({ task }: IProgressTaskItem) => {
    const percent = getProgressPercent(task);
    return (
        <div className="rounded border border-gray-200 bg-white p-3 shadow-sm">
            {/* Header */}
            <div className="mb-2 flex items-center justify-between gap-3">
                <div className="min-w-0">
                    <div className="text-sm font-semibold text-gray-800">
                        {TASK_LABELS[task.task]}
                    </div>

                    <div className="text-xs text-gray-500">
                        MST: {task.tax_code}
                    </div>
                </div>
                <Loading />
            </div>

            {/* Progress */}
            <div className="mb-2 flex items-center justify-between text-xs text-gray-600">
                <span>{task.message}</span>

                <span className="shrink-0 font-medium tabular-nums">
                    {task.total == null
                        ? `${task.completed} đã xử lý`
                        : `${task.completed}/${task.total}`}
                </span>
            </div>

            <div className="h-1.5 w-full overflow-hidden rounded-full bg-gray-200">
                {percent === null ? (
                    <div className="h-full w-1/3 animate-pulse rounded-full bg-blue-500" />
                ) : (
                    <div
                        className="h-full rounded-full bg-blue-500 transition-all duration-300"
                        style={{ width: `${percent}%` }}
                    />
                )}
            </div>

            {/* Current invoice */}
            {(task.invoice_number != null ||
                task.invoice_serial ||
                task.invoice_date) && (
                    <div className="mt-2 text-xs text-gray-500">
                        {task.invoice_serial && (
                            <span>Ký hiệu: {task.invoice_serial} </span>
                        )}

                        {task.invoice_number != null && (
                            <span>Số hóa đơn: {task.invoice_number} </span>
                        )}

                        {task.invoice_date && (
                            <span>Ngày: {task.invoice_date}</span>
                        )}
                    </div>
                )}

            {/* Error */}
            {task.error && (
                <div className="mt-2 text-xs text-red-600">
                    {task.error}
                </div>
            )}
        </div>
    );
}

function getProgressPercent(progress: TaskProgress): number | null {
    if (progress.total == null || progress.total <= 0) {
        return null;
    }

    return Math.min(
        100,
        Math.round((progress.completed / progress.total) * 100)
    );
}
