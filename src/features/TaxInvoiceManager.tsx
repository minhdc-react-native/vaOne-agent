
import { useCallback, useEffect, useMemo, useState } from "react";
import {
    useGetInvoiceStats,
    useGetSyncInvoices,
} from "../hook/useSyncInvoices";
import type {
    IInvoiceStats,
    ISyncInvoiceFilters,
    IPaginatedResult,
    ISyncInvoice,
} from "../service/syncInvoicesService";
import { useTaxAccounts } from "../hook/useTaxAccounts";
import StatusBadge, { StatusType } from "../components/StatusBadge";
import { useNavigate, useParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import AppWindow from "../components/AppWindow";
import { ComboPopup } from "../components/Select/ComboPopup";
import { DateInput } from "../components/Date/DateInput";
import { formatNumber, getCurrentDateTime } from "../lib/utils";
import { ChevronLeft, ChevronRight } from "lucide-react";
import Button from "../components/Button";
import { formatDate } from "../api/services/tct.service";


const PAGE_SIZE = 20;

const INVOICE_TYPES = [
    { id: 1, label: "Hóa đơn mua vào" },
    { id: 2, label: "Hóa đơn bán ra" },
    { id: 3, label: "Mua vào MTT" },
    { id: 4, label: "Bán ra MTT" },
];
const mapInvoiceType: Record<number, string> = {
    1: "Hóa đơn mua vào",
    2: "Hóa đơn bán ra",
    3: "Mua vào MTT",
    4: "Bán ra MTT"
}

const mapStatus: Record<string, StatusType> = {
    pending: "pending",
    running: "continue",
    completed: "success",
    failed: "danger"
}
const DETAIL_STATUSES = [
    { id: "pending", label: "Chờ lấy" },
    { id: "running", label: "Đang lấy" },
    { id: "completed", label: "Đã lấy" },
    { id: "failed", label: "Lỗi lấy" },
];

const UPLOAD_STATUSES = [
    { id: "pending", label: "Chờ đồng bộ" },
    { id: "running", label: "Đang đồng bộ" },
    { id: "completed", label: "Đã đồng bộ" },
    { id: "failed", label: "Lỗi đồng bộ" },
];
const mapStatusLabel: Record<string, string> = {
    pending: "Chờ thực hiện",
    running: "Đang thực hiện",
    completed: "Hoàn thành",
    failed: "Lỗi"
}
const initialFilters: ISyncInvoiceFilters = {
    tax_code: "",
    invoice_type: 1,
    from_date: getCurrentDateTime(),
    to_date: getCurrentDateTime(),
    detail_status: undefined,
    upload_status: undefined,
};

// function formatDate(value?: string | null) {
//     if (!value) return "—";

//     const date = new Date(
//         value.length === 10 ? `${value}T00:00:00` : value,
//     );

//     if (Number.isNaN(date.getTime())) return value;

//     return new Intl.DateTimeFormat("vi-VN").format(date);
// }

// function formatDateTime(value?: string | null) {
//     if (!value) return "—";

//     const date = new Date(value);

//     if (Number.isNaN(date.getTime())) return value;

//     return new Intl.DateTimeFormat("vi-VN", {
//         dateStyle: "short",
//         timeStyle: "short",
//     }).format(date);
// }

function StatCard({
    label,
    value,
}: {
    label: string;
    value: number;
}) {
    return (
        <div className="min-w-0 rounded-xl border border-gray-200 bg-white p-2">
            <div className="truncate text-sm text-gray-500">{label}</div>
            <div className="mt-1 text-lg font-semibold tabular-nums">
                {value.toLocaleString("vi-VN")}
            </div>
        </div>
    );
}

export function TaxInvoiceManager() {
    const { taxCode } = useParams<{ taxCode: string }>();
    const {
        accounts,
    } = useTaxAccounts();

    const account = useMemo(() => accounts.find(acc => acc.tax_code === taxCode), [taxCode]);

    const {
        mutateAsync: getStats,
        isPending: statsLoading,
    } = useGetInvoiceStats();

    const {
        mutateAsync: getInvoices,
        isPending: invoicesLoading,
    } = useGetSyncInvoices();

    const [stats, setStats] = useState<IInvoiceStats | null>(null);

    const [invoicePage, setInvoicePage] =
        useState<IPaginatedResult<ISyncInvoice> | null>(null);

    const [page, setPage] = useState(1);

    const [filters, setFilters] = useState<ISyncInvoiceFilters>({
        ...initialFilters,
        tax_code: taxCode,
    });

    const [localError, setLocalError] = useState<string | null>(null);

    const loadStats = useCallback(async () => {
        if (!taxCode) return;

        try {
            const result = await getStats(taxCode);
            setStats(result);
        } catch (error) {
            setLocalError(String(error));
        }
    }, [taxCode, getStats]);

    const loadInvoices = useCallback(
        async (
            nextFilters: ISyncInvoiceFilters,
            nextPage: number,
        ) => {
            if (!taxCode) return;

            try {
                setLocalError(null);
                const result = await getInvoices({
                    filters: {
                        ...nextFilters,
                        tax_code: taxCode,
                    },
                    page: nextPage,
                    page_size: PAGE_SIZE,
                });

                setInvoicePage(result);
                setPage(nextPage);
            } catch (error) {
                setLocalError(String(error));
            }
        },
        [taxCode, getInvoices],
    );

    // Tải dữ liệu khi tax_code thay đổi.
    useEffect(() => {
        setStats(null);
        setInvoicePage(null);
        setPage(1);

        setFilters({
            ...initialFilters,
            tax_code: taxCode
        });

        if (!taxCode) return;

        void loadStats();
        void loadInvoices(initialFilters, 1);
    }, [taxCode]); // eslint-disable-line react-hooks/exhaustive-deps

    const updateFilter = <K extends keyof ISyncInvoiceFilters>(
        key: K,
        value: ISyncInvoiceFilters[K],
    ) => {
        setFilters((current) => ({
            ...current,
            [key]: value,
        }));
    };

    const applyFilters = () => {
        void loadInvoices(filters, 1);
    };

    useEffect(() => {
        applyFilters();
    }, [filters])

    const totalPages = invoicePage?.total_pages ?? 0;
    const invoiceItems = invoicePage?.items ?? [];

    const navigate = useNavigate();
    useEffect(() => {
        invoke("page_ready", { name: "historyInvoice", show: true });
    }, []);

    const onClose = useCallback(() => {
        navigate("/tax-config");
    }, [])


    return (
        <AppWindow icon="History"
            content={
                <div data-tauri-drag-region className="cursor-pointer flex-1 flex flex-wrap items-center justify-between gap-3">
                    <div data-tauri-drag-region className="mt-1 text-sm font-semibold">
                        <span data-tauri-drag-region className="text-red-600">{taxCode}</span>
                        {" - "}{account?.note}
                    </div>
                    {/* <Button
                        variant="ghost"
                        icon={<RefreshCcw />}
                        onClick={() => void refreshAll()}
                        disabled={statsLoading || invoicesLoading}
                    >
                        Làm mới dữ liệu
                    </Button> */}
                </div>
            }
            onClose={onClose}
        >
            <div className="flex h-full min-h-0 w-full min-w-0 flex-col gap-4 text-sm text-gray-800 p-2 bg-gray-100">
                <div className="flex gap-1">
                    <div className="grid grid-cols-2 gap-1 w-70 shrink-0">
                        <StatCard
                            label="Tổng hóa đơn"
                            value={stats?.total ?? 0}
                        />

                        <StatCard
                            label="Chờ lấy chi tiết"
                            value={stats?.groups
                                .filter((item) => item.detail_status === "pending")
                                .reduce((sum, item) => sum + item.count, 0) ?? 0}
                        />

                        <StatCard
                            label="Lỗi lấy chi tiết"
                            value={stats?.groups
                                .filter((item) => item.detail_status === "failed")
                                .reduce((sum, item) => sum + item.count, 0) ?? 0}
                        />

                        <StatCard
                            label="Lỗi upload"
                            value={stats?.groups
                                .filter((item) => item.upload_status === "failed")
                                .reduce((sum, item) => sum + item.count, 0) ?? 0}
                        />
                    </div>

                    {/* Filters */}
                    <section className="flex-1 rounded border border-gray-200 bg-white p-3 space-y-1">
                        <div className="mb-3 font-semibold">Bộ lọc hóa đơn</div>

                        <div className="flex gap-1 flex-1">
                            <div className="flex-1 flex gap-1 items-center">
                                <span className="w-12">Loại</span>
                                <ComboPopup
                                    value={INVOICE_TYPES.find(i => i.id === filters.invoice_type)}
                                    data={INVOICE_TYPES}
                                    className="w-45 h-7"
                                    onClear={() => {
                                        updateFilter(
                                            "invoice_type",
                                            undefined,
                                        );
                                    }}
                                    onSelect={(item: any) => {
                                        updateFilter(
                                            "invoice_type",
                                            item.id,
                                        )
                                    }}
                                    renderItem={(item) => <div key={item?.id} className="px-2 w-40">{item?.label}</div>}
                                />
                            </div>
                            <div className="flex-1 w-50">
                                <DateInput
                                    label="Từ ngày"
                                    labelPosition="left"
                                    required
                                    value={filters.from_date ?? ""}
                                    onChange={(value) =>
                                        updateFilter(
                                            "from_date",
                                            value || undefined,
                                        )
                                    }
                                />
                            </div>
                            <div className="w-50">
                                <DateInput
                                    label="Đến ngày"
                                    labelPosition="left"
                                    required
                                    value={filters.to_date ?? ""}
                                    onChange={(value) =>
                                        updateFilter(
                                            "to_date",
                                            value || undefined,
                                        )
                                    }
                                />
                            </div>
                        </div>
                        <div className="flex gap-1">
                            <div className="flex-1 flex gap-1 items-center">
                                <span className="w-12">Chi tiết</span>
                                <ComboPopup
                                    value={DETAIL_STATUSES.find(i => i.id === filters.detail_status)}
                                    data={DETAIL_STATUSES}
                                    className="w-45 h-7"
                                    onClear={() => {
                                        updateFilter(
                                            "detail_status",
                                            undefined,
                                        );
                                    }}
                                    onSelect={(item: any) => {
                                        updateFilter(
                                            "detail_status",
                                            item.id,
                                        )
                                    }}
                                    renderItem={(item) => <div key={item?.id} className="px-2 w-40">{item?.label}</div>}
                                />
                            </div>

                            <div className="flex-2 flex gap-1 items-center">
                                <span className="w-18">Đồng bộ</span>
                                <ComboPopup
                                    value={UPLOAD_STATUSES.find(i => i.id === filters.upload_status)}
                                    data={UPLOAD_STATUSES}
                                    className="w-39 h-7"
                                    onClear={() => {
                                        updateFilter(
                                            "upload_status",
                                            undefined,
                                        );
                                    }}
                                    onSelect={(item: any) => {
                                        updateFilter(
                                            "upload_status",
                                            item.id,
                                        )
                                    }}
                                    renderItem={(item) => <div key={item?.id} className="px-2 w-40">{item?.label}</div>}
                                />
                            </div>
                        </div>
                        {/* <div className="flex items-end gap-2 flex-1">
                            <Button
                                variant="ghost"
                                icon={<Filter size={15} />}
                                onClick={applyFilters}
                                disabled={statsLoading || invoicesLoading}
                            >
                                Lọc dữ liệu
                            </Button>
                        </div> */}
                        {localError && (
                            <div className="mt-3 rounded bg-red-50 p-2 text-red-600">
                                {localError}
                            </div>
                        )}
                    </section>
                </div>
                {/* Main content: 1/3 sync days + 2/3 invoices */}
                <div className="flex-1 grid min-h-0 grid-cols-1 gap-4 xl:grid-cols-3">
                    <section className="min-w-0 min-h-0 flex flex-col overflow-hidden rounded border border-gray-200 bg-white xl:col-span-2">
                        {/* Header: giữ nguyên chiều cao */}
                        <div className="flex shrink-0 flex-wrap items-center justify-between gap-2 border-b border-gray-200 px-3 py-3">
                            <h3 className="font-semibold">Danh sách hóa đơn</h3>

                            <span className="text-xs text-gray-500">
                                {(invoicePage?.total ?? 0).toLocaleString("vi-VN")} hóa đơn
                            </span>
                        </div>

                        {/* Chỉ chiếm phần không gian còn lại */}
                        <div className="flex-1 min-h-0 overflow-auto">
                            {invoicesLoading ? (
                                <div className="p-4 text-gray-500">
                                    Đang tải danh sách hóa đơn...
                                </div>
                            ) : invoiceItems.length === 0 ? (
                                <div className="p-4 text-gray-500">
                                    Không có hóa đơn phù hợp.
                                </div>
                            ) : (
                                <div className="min-w-full w-max">
                                    <table className="w-full min-w-max border-collapse text-left">
                                        <thead className="sticky top-0 z-10 bg-gray-50">
                                            <tr>
                                                <th className="w-14 px-3 py-2 font-medium text-center">#</th>

                                                <th className="w-32 px-3 py-2 font-medium text-center">
                                                    Ngày hóa đơn
                                                </th>

                                                <th className="w-36 px-3 py-2 font-medium text-center">
                                                    Loại
                                                </th>

                                                <th className="w-32 px-3 py-2 font-medium text-center">
                                                    Ký hiệu
                                                </th>

                                                <th className="w-32 px-3 py-2 font-medium text-right">
                                                    Số hóa đơn
                                                </th>

                                                <th className="w-35 px-3 py-2 font-medium text-center">
                                                    Chi tiết
                                                </th>

                                                <th className="w-35 px-3 py-2 font-medium text-center">
                                                    Đồng bộ
                                                </th>
                                                <th className="w-28 px-3 py-2 font-medium text-center">
                                                    Tiền tệ
                                                </th>
                                                <th className="w-28 px-3 py-2 font-medium text-center">
                                                    Tiền trước thuế
                                                </th>
                                                <th className="w-28 px-3 py-2 font-medium text-center">
                                                    Thuế
                                                </th>
                                                <th className="w-28 px-3 py-2 font-medium text-center">
                                                    Tổng tiền
                                                </th>
                                            </tr>
                                        </thead>

                                        <tbody>
                                            {invoiceItems.map((invoice, index) => (
                                                <tr
                                                    key={`${invoice.tax_code}-${invoice.invoice_type}-${invoice.invoice_key}`}
                                                    className="border-t border-gray-100 hover:bg-gray-50"
                                                >
                                                    <td className="whitespace-nowrap px-3 py-2">
                                                        {index + 1 + (PAGE_SIZE * (page - 1))}
                                                    </td>
                                                    <td className="whitespace-nowrap px-3 py-2 text-center">
                                                        {formatDate(invoice.invoice_date)}
                                                    </td>

                                                    <td className="whitespace-nowrap px-3 py-2">
                                                        {mapInvoiceType[invoice.invoice_type]}
                                                    </td>

                                                    <td className="whitespace-nowrap px-3 py-2">
                                                        {invoice.khhdon ?? "—"}
                                                    </td>

                                                    <td className="whitespace-nowrap px-3 py-2 text-right">
                                                        {invoice.shdon ?? "—"}
                                                    </td>

                                                    <td className="px-3 py-2">
                                                        <StatusBadge
                                                            type={mapStatus[invoice.detail_status]}
                                                            label={mapStatusLabel[invoice.detail_status]}
                                                        />
                                                    </td>

                                                    <td className="px-3 py-2">
                                                        <StatusBadge
                                                            type={mapStatus[invoice.upload_status]}
                                                            label={mapStatusLabel[invoice.upload_status]}
                                                        />
                                                    </td>
                                                    <td className="whitespace-nowrap px-3 py-2">
                                                        {invoice.dvtte ?? "—"}
                                                    </td>
                                                    <td className="whitespace-nowrap px-3 py-2 text-right">
                                                        {formatNumber(invoice.tgtcthue, invoice.tgia !== 1 ? 2 : 0)}
                                                    </td>
                                                    <td className="whitespace-nowrap px-3 py-2 text-right">
                                                        {formatNumber(invoice.tgtthue, invoice.tgia !== 1 ? 2 : 0)}
                                                    </td>
                                                    <td className="whitespace-nowrap px-3 py-2 text-right">
                                                        {formatNumber(invoice.tgtttbso, invoice.tgia !== 1 ? 2 : 0)}
                                                    </td>
                                                </tr>
                                            ))}
                                        </tbody>
                                    </table>
                                </div>
                            )}
                        </div>

                        {/* Pagination */}
                        <div className="flex flex-wrap items-center justify-between gap-3 border-t border-gray-200 px-3 py-3">
                            <span className="text-xs text-gray-500">
                                Trang {invoicePage?.page ?? page}
                                {totalPages > 0 ? ` / ${totalPages}` : ""}
                                {" · "}
                                {PAGE_SIZE} dòng/trang
                            </span>

                            <div className="flex gap-2">
                                <Button
                                    iconOnly={true}
                                    variant="ghost"
                                    icon={<ChevronLeft size={15} />}
                                    disabled={
                                        invoicesLoading ||
                                        page <= 1
                                    }
                                    onClick={() =>
                                        void loadInvoices(filters, page - 1)
                                    }
                                />
                                <Button
                                    iconOnly={true}
                                    variant="ghost"
                                    icon={<ChevronRight size={15} />}
                                    disabled={
                                        invoicesLoading ||
                                        totalPages === 0 ||
                                        page >= totalPages
                                    }
                                    onClick={() =>
                                        void loadInvoices(filters, page + 1)
                                    }
                                />
                            </div>
                        </div>
                    </section>
                </div>
            </div>
        </AppWindow>
    );
}
