
import { useCallback, useEffect, useMemo, useState } from "react";
import {
    useGetInvoiceStats,
    useGetSyncInvoices,
    useResetSyncStatus,
} from "../hook/useSyncInvoices";
import type {
    IInvoiceStats,
    ISyncInvoiceFilters,
    IPaginatedResult,
    ISyncInvoice,
    ISyncConfig,
} from "../service/syncInvoicesService";
import { useTaxAccounts } from "../hook/useTaxAccounts";
import StatusBadge, { StatusType } from "../components/StatusBadge";
import { useNavigate, useParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import AppWindow from "../components/AppWindow";
import { ComboPopup } from "../components/Select/ComboPopup";
import { DateInput } from "../components/Date/DateInput";
import { formatNumber, getCurrentMonthDateRange } from "../lib/utils";
import { ChevronLeft, ChevronRight, Clock, CloudSync, Loader, RefreshCcw } from "lucide-react";
import Button from "../components/Button";
import { formatDate } from "../api/services/tct.service";
import { dialog } from "../service/dialog.service";
import { Tabs } from "../components/Tabs";
import Switch from "../components/Switch";
import { showNotification } from "../lib/notification";
import clsx from "clsx";
import Input from "../components/Input";


const PAGE_SIZE = 100;

const INVOICE_TYPES = [
    { id: 1, label: "Hóa đơn mua vào" },
    { id: 2, label: "Hóa đơn bán ra" },
    { id: 3, label: "Mua vào MTT" },
    { id: 4, label: "Bán ra MTT" },
];
const mapInvoiceType: Record<number, string> = {
    1: "Mua vào",
    2: "Bán ra",
    3: "Mua vào",
    4: "Bán ra"
}

const mapInvoiceStatus: Record<number, string> = {
    1: "Hóa đơn mới",
    2: "Hóa đơn thay thế",
    3: "Hóa đơn điều chỉnh",
    4: "Hóa đơn đã bị thay thế",
    5: "Hóa đơn đã bị điều chỉnh"
}
const mapInvoiceStyle: Record<number, StatusType> = {
    1: "pending",
    2: "warning",
    3: "progress",
    4: "draft",
    5: "draft"
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
const dateMonth = getCurrentMonthDateRange();
const initialFilters: ISyncInvoiceFilters = {
    tax_code: "",
    nbmst: undefined,
    khhdon: undefined,
    shdon: undefined,
    invoice_type: 1,
    from_date: dateMonth.fromDate,
    to_date: dateMonth.toDate,
    detail_status: undefined,
    upload_status: undefined,
};
const initialSync: ISyncConfig = {
    taxCode: "",
    fromDate: dateMonth.fromDate,
    toDate: dateMonth.toDate,
    taxSync: false,
    appSync: true,
};

function StatCard({
    label,
    value,
    classNumber
}: {
    label: string;
    value: number;
    classNumber?: string;
}) {
    return (
        <div className="w-full rounded-xl border border-gray-200 bg-white p-2">
            <div className="truncate text-sm text-gray-500">{label}</div>
            <div className={clsx("mt-1 text-3xl font-semibold tabular-nums text-right", classNumber)}>
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

    const { mutate: resetSyncStatus, isPending } = useResetSyncStatus();

    const [syncConfig, setSyncConfig] = useState({
        ...initialSync,
        taxCode: taxCode ?? ""
    });

    const [filters, setFilters] = useState<ISyncInvoiceFilters>({
        ...initialFilters,
        tax_code: taxCode,
    });


    const [stats, setStats] = useState<IInvoiceStats | null>(null);

    const [invoicePage, setInvoicePage] =
        useState<IPaginatedResult<ISyncInvoice> | null>(null);

    const [page, setPage] = useState(1);


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

    const updateSyncConfig = <K extends keyof ISyncConfig>(
        key: K,
        value: ISyncConfig[K],
    ) => {
        setSyncConfig((current) => ({
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

    const handleReset = async () => {
        if (!filters.from_date || !filters.to_date) {
            showNotification({
                type: "warning",
                message: "Bạn phải nhập đủ ngày!"
            });
            return;
        };
        const confirmed = await dialog.question(
            `Bạn có chắc chắn đồng bộ lại dữ liệu của mã số thuế ${taxCode}?`,
            `Từ ngày: ${formatDate(filters.from_date)} đến ngày ${formatDate(filters.to_date)}`,
        );
        if (!confirmed) return;
        resetSyncStatus({
            ...syncConfig,
            fromDate: filters.from_date,
            toDate: filters.to_date
        },
            {
                onSuccess: () => {
                    applyFilters();
                    showNotification({ type: "success", message: "Cập nhật thành công" });
                },
                onError: (error) => {
                    console.error("Reset sync status failed:", error);
                    showNotification({ type: "error", message: "Không thể cập nhật" })
                },
            },
        );
    };


    const totalPages = invoicePage?.total_pages ?? 0;
    const invoiceItems = invoicePage?.items ?? [];

    const navigate = useNavigate();
    useEffect(() => {
        invoke("page_ready", { name: "historyInvoice", show: true });
    }, []);

    const onClose = useCallback(() => {
        navigate(`/tax-config/${taxCode}`);
    }, [])

    useEffect(() => {
        if (localError) dialog.info(
            localError,
            "Thông tin lỗi",
        );
    }, [localError]);
    const [tab, setTab] = useState(0);
    return (
        <AppWindow icon="History"
            content={
                <div data-tauri-drag-region className="cursor-pointer flex-1 flex flex-wrap items-center justify-between gap-3">
                    <div data-tauri-drag-region className="mt-1 text-sm font-semibold">
                        <span data-tauri-drag-region className="text-red-600">{taxCode}</span>
                        {" - "}{account?.note}
                    </div>
                    <Button
                        variant="ghost"
                        className="h-6"
                        icon={<RefreshCcw size={18} />}
                        onClick={() => void applyFilters()}
                        disabled={statsLoading || invoicesLoading}
                    >
                        Làm mới dữ liệu
                    </Button>
                </div>
            }
            onClose={onClose}
        >
            <div className="flex h-full min-h-0 w-full min-w-0 flex-col gap-4 text-sm text-gray-800 p-2 bg-gray-100">
                <div className="flex gap-1">
                    <div className="flex flex-col gap-1 w-70 shrink-0">
                        <div className="flex items-center gap-2 rounded-xl border border-gray-200 bg-white px-3 py-2">
                            <Clock className="h-6 w-6 shrink-0 text-gray-500" />

                            <span className="text-sm text-gray-500">
                                Đồng bộ
                            </span>

                            <span className="text-sm font-medium text-gray-800">
                                {account?.updated_at
                                    ? formatDate(account.updated_at, true)
                                    : "Chưa đồng bộ"}
                            </span>
                        </div>
                        <div className="grid grid-cols-2 gap-1 flex-1">
                            <StatCard
                                label="Tổng hóa đơn"
                                value={stats?.total ?? 0}
                                classNumber="text-green-600"
                            />

                            <StatCard
                                label="Chờ lấy chi tiết"
                                value={stats?.groups
                                    .filter((item) => item.detail_status === "pending")
                                    .reduce((sum, item) => sum + item.count, 0) ?? 0}
                                classNumber="text-blue-600"
                            />

                            <StatCard
                                label="Lỗi lấy chi tiết"
                                value={stats?.groups
                                    .filter((item) => item.detail_status === "failed")
                                    .reduce((sum, item) => sum + item.count, 0) ?? 0}
                                classNumber="text-orange-600"
                            />

                            <StatCard
                                label="Lỗi upload"
                                value={stats?.groups
                                    .filter((item) => item.upload_status === "failed")
                                    .reduce((sum, item) => sum + item.count, 0) ?? 0}
                                classNumber="text-orange-600"
                            />
                        </div>
                    </div>
                    <section className="flex-1 rounded border border-gray-200 bg-white p-3 space-y-1">
                        <Tabs variant="line" defaultActiveTab={tab} onChange={(idx) => setTab(idx)}>
                            <Tabs.Header>
                                <Tabs.Tab index={0} labelClassActive="font-semibold">{"Bộ lọc hóa đơn"}</Tabs.Tab>
                                <Tabs.Tab index={1} labelClassActive="font-semibold">
                                    Cập nhật đồng bộ
                                </Tabs.Tab>
                            </Tabs.Header>
                            <Tabs.Content className="border-0">
                                <Tabs.Panel index={0} className="border border-gray-300 rounded-md">
                                    {/* Filters */}
                                    <div className="flex-1 bg-white p-3 space-y-1 h-28 w-100">
                                        <div className="flex gap-1 flex-1">
                                            <div className="flex-1 flex gap-1 items-center">
                                                <span className="w-12 shrink-0">Loại</span>
                                                <ComboPopup
                                                    value={INVOICE_TYPES.find(i => i.id === filters.invoice_type)}
                                                    data={INVOICE_TYPES}
                                                    className="w-45 shrink-0 h-7"
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
                                                    renderItem={(item) => <div key={item?.id} className="px-2 w-40 shrink-0">{item?.label}</div>}
                                                />
                                            </div>
                                            <div className="w-50 shrink-0">
                                                <DateInput
                                                    label="Từ ngày"
                                                    labelPosition="left"
                                                    value={filters.from_date ?? ""}
                                                    onChange={(value) =>
                                                        updateFilter(
                                                            "from_date",
                                                            value || undefined,
                                                        )
                                                    }
                                                />
                                            </div>
                                            <div className="w-50 shrink-0">
                                                <DateInput
                                                    label="Đến ngày"
                                                    labelPosition="left"
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
                                                <span className="w-12 shrink-0">Chi tiết</span>
                                                <ComboPopup
                                                    value={DETAIL_STATUSES.find(i => i.id === filters.detail_status)}
                                                    data={DETAIL_STATUSES}
                                                    className="w-45 shrink-0 h-7"
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
                                                    renderItem={(item) => <div key={item?.id} className="px-2 w-40 shrink-0">{item?.label}</div>}
                                                />
                                            </div>

                                            <div className="flex-2 flex gap-1 items-center">
                                                <span className="w-17.5 shrink-0">Đồng bộ</span>
                                                <ComboPopup
                                                    value={UPLOAD_STATUSES.find(i => i.id === filters.upload_status)}
                                                    data={UPLOAD_STATUSES}
                                                    className="w-39 shrink-0 h-7"
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
                                                    renderItem={(item) => <div key={item?.id} className="px-2 w-40 shrink-0">{item?.label}</div>}
                                                />
                                            </div>
                                        </div>
                                        <div className="flex gap-1 flex-1">
                                            <div className="flex-1 w-58 shrink-0">
                                                <Input
                                                    label="MST"
                                                    labelPosition="left"
                                                    labelWidth={53}
                                                    value={filters.nbmst ?? ""}
                                                    onChange={(e) =>
                                                        updateFilter(
                                                            "nbmst",
                                                            e.target.value || undefined,
                                                        )
                                                    }
                                                />
                                            </div>
                                            <div className="w-57.5 shrink-0">
                                                <Input
                                                    label="Ký hiệu"
                                                    labelPosition="left"
                                                    labelWidth={98}
                                                    value={filters.khhdon ?? ""}
                                                    onChange={(e) =>
                                                        updateFilter(
                                                            "khhdon",
                                                            e.target.value || undefined,
                                                        )
                                                    }
                                                />
                                            </div>
                                            <div className="w-55 shrink-0">
                                                <Input
                                                    label="Số hóa đơn"
                                                    type="number"
                                                    labelPosition="left"
                                                    className="text-right"
                                                    value={filters.shdon ?? ""}
                                                    onChange={(e) =>
                                                        updateFilter(
                                                            "shdon",
                                                            Number(e.target.value) || undefined,
                                                        )
                                                    }
                                                />
                                            </div>
                                        </div>
                                    </div>
                                </Tabs.Panel>
                                <Tabs.Panel index={1} className="border border-gray-300 rounded-md">
                                    <div className="flex-1 bg-white p-3 space-y-2 h-28 w-100">
                                        <div className="flex gap-1 flex-1">
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
                                            <div className="flex-1" />
                                        </div>
                                        <div className="flex gap-4 items-center">
                                            <div className="flex flex-col gap-2">
                                                <Switch
                                                    label="Lấy lại từ cơ quan thuế"
                                                    // description="Cho phép worker xử lý tài khoản này"
                                                    checked={syncConfig.taxSync}
                                                    onChange={(value) =>
                                                        updateSyncConfig("taxSync", value)
                                                    }
                                                />
                                                <Switch
                                                    label="Đồng bộ lại sang ứng dụng"
                                                    // description="Cho phép worker xử lý tài khoản này"
                                                    checked={syncConfig.appSync}
                                                    onChange={(value) =>
                                                        updateSyncConfig("appSync", value)
                                                    }
                                                />
                                            </div>
                                            <Button onClick={handleReset} icon={isPending ? <Loader className="animate-spin" /> : <CloudSync />} variant="danger">
                                                Đồng bộ lại
                                            </Button>
                                        </div>
                                    </div>
                                </Tabs.Panel>
                            </Tabs.Content>
                        </Tabs>
                    </section>
                </div>
                {/* Main content: 1/3 sync days + 2/3 invoices */}
                <div className="flex-1 grid min-h-0 grid-cols-1 gap-4">
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
                                        <thead className="sticky top-0 z-10 bg-gray-100">
                                            <tr>
                                                <th className="w-14 px-2 py-1 font-medium">#</th>
                                                <th className="w-40 px-2 py-1 font-medium">
                                                    Mã số thuế
                                                </th>

                                                <th className="w-36 px-2 py-1 font-medium">
                                                    Loại
                                                </th>

                                                <th className="w-32 px-2 py-1 font-medium">
                                                    Ký hiệu
                                                </th>

                                                <th className="w-32 px-2 py-1 font-medium text-right">
                                                    Số hóa đơn
                                                </th>
                                                <th className="w-32 px-2 py-1 font-medium text-center">
                                                    Ngày lập
                                                </th>
                                                <th className="px-2 py-1 font-medium">
                                                    Tên người bán
                                                </th>
                                                <th className="w-28 px-2 py-1 font-medium">
                                                    Tiền tệ
                                                </th>
                                                <th className="w-28 px-2 py-1 font-medium text-right">
                                                    Tiền hàng
                                                </th>
                                                <th className="w-28 px-2 py-1 font-medium text-right">
                                                    Thuế
                                                </th>
                                                <th className="w-28 px-2 py-1 font-medium text-right">
                                                    Tổng tiền
                                                </th>
                                                <th className="w-35 px-2 py-1 font-medium text-center">
                                                    Dữ liệu
                                                </th>
                                                <th className="w-35 px-2 py-1 font-medium text-center">
                                                    Đồng bộ
                                                </th>
                                                <th className="w-50 px-2 py-1 font-medium text-center">
                                                    Trạng thái
                                                </th>
                                            </tr>
                                        </thead>

                                        <tbody>
                                            {invoiceItems.map((invoice, index) => (
                                                <tr
                                                    key={`${invoice.tax_code}-${invoice.invoice_type}-${invoice.invoice_key}`}
                                                    className="border-t border-gray-100 hover:bg-gray-50 font-xs"
                                                >
                                                    <td className="whitespace-nowrap px-2 py-1">
                                                        {index + 1 + (PAGE_SIZE * (page - 1))}
                                                    </td>

                                                    <td className="whitespace-nowrap px-2 py-1">
                                                        {invoice.nbmst}
                                                    </td>

                                                    <td className="whitespace-nowrap px-2 py-1">
                                                        {mapInvoiceType[invoice.invoice_type]}
                                                    </td>

                                                    <td className="whitespace-nowrap px-2 py-1">
                                                        {invoice.khhdon ?? "—"}
                                                    </td>

                                                    <td className="whitespace-nowrap px-2 py-1 text-right">
                                                        {invoice.shdon ?? "—"}
                                                    </td>
                                                    <td className="whitespace-nowrap px-2 py-1 text-center">
                                                        {formatDate(invoice.invoice_date)}
                                                    </td>
                                                    <td className="whitespace-nowrap px-2 py-1">
                                                        {invoice.nbten}
                                                    </td>

                                                    <td className="whitespace-nowrap px-2 py-1">
                                                        {invoice.dvtte ?? "—"}
                                                    </td>
                                                    <td className="whitespace-nowrap px-2 py-1 text-right">
                                                        {formatNumber(invoice.tgtcthue, invoice.tgia !== 1 ? 2 : 0)}
                                                    </td>
                                                    <td className="whitespace-nowrap px-2 py-1 text-right">
                                                        {formatNumber(invoice.tgtthue, invoice.tgia !== 1 ? 2 : 0)}
                                                    </td>
                                                    <td className="whitespace-nowrap px-2 py-1 text-right">
                                                        {formatNumber(invoice.tgtttbso, invoice.tgia !== 1 ? 2 : 0)}
                                                    </td>
                                                    <td className="px-2 py-1 text-center">
                                                        <StatusBadge
                                                            type={mapStatus[invoice.detail_status]}
                                                            label={mapStatusLabel[invoice.detail_status]}
                                                        />
                                                    </td>

                                                    <td className="px-2 py-1 text-center">
                                                        <StatusBadge
                                                            type={mapStatus[invoice.upload_status]}
                                                            label={mapStatusLabel[invoice.upload_status]}
                                                        />
                                                    </td>
                                                    <td className="px-2 py-1 text-center">
                                                        <StatusBadge
                                                            variant="text"
                                                            type={mapInvoiceStyle[invoice.tthai as any]}
                                                            label={mapInvoiceStatus[invoice.tthai as any]}
                                                        />
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
