import { formatDate } from "../api/services/tct.service";
import StatusBadge, { StatusType } from "../components/StatusBadge";
import { MaybeTooltip } from "../components/Tooltip/MaybeTooltip";
import { ISyncDay } from "../service/syncInvoicesService";

const mapInvoiceType: Record<number, string> = {
    1: "Mua vào",
    2: "Bán ra",
    3: "Mua vào MTT",
    4: "Bán ra MTT"
}

const mapStatus: Record<string, StatusType> = {
    pending: "pending",
    running: "continue",
    completed: "success",
    failed: "danger"
}
const mapStatusLabel: Record<string, string> = {
    pending: "Chờ thực hiện",
    running: "Đang thực hiện",
    completed: "Hoàn thành",
    failed: "Lỗi"
}

interface ISyncDaysError {
    item?: ISyncDay | null
}
export const SyncDaysError = ({ item }: ISyncDaysError) => {
    if (!item) return null;
    return (
        <div key={item.id} className="flex items-center gap-2 px-3 py-2 text-xs">
            <span className="font-medium">{formatDate(item.sync_date)}</span>

            <span className="text-gray-500">
                {mapInvoiceType[item.invoice_type]}
            </span>

            <MaybeTooltip enabled={!!item.last_error} tooltip={item.last_error!}>
                <StatusBadge
                    type={mapStatus[item.list_status]}
                    label={mapStatusLabel[item.list_status]}
                />
            </MaybeTooltip>
        </div>
    );
}