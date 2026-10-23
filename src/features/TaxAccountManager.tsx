
import { ReactNode, useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
    Building2,
    Check,
    CircleAlert,
    Clock,
    History,
    KeyRound,
    List,
    Loader,
    MessageCircleMore,
    MessageCircleWarning,
    Pencil,
    Plus,
    RefreshCw,
    Save,
    Trash2,
    X,
} from "lucide-react";
import Input from "../components/Input";
import { DateInput } from "../components/Date/DateInput";
import Switch from "../components/Switch";
import Button from "../components/Button";
import { dialog } from "../service/dialog.service";
import { useCheckLoginApp, useCheckTaxAccountLogin, useTaxAccounts } from "../hook/useTaxAccounts";
import { ITaxAccount, ITaxAccountInput, TypeApp } from "../service/taxAccountService";
import { Loading } from "../components/Loading";
import InvoiceTypeCheckbox from "./InvoiceTypeCheckbox";
import { Tooltip } from "../components/Tooltip/Tooltip";
import { showNotification } from "../lib/notification";
import { useNavigate, useParams } from "react-router-dom";
import { useVirtualized } from "../hook/useVirtualized";
import { formatDate } from "../api/services/tct.service";
import { useLatest } from "../hook/useLatest";
import { usePauseTaxSyncWorker } from "../hook/usePauseTaxSyncWorker";
import { isEmpty } from "../lib/utils";
import { ComboPopup } from "../components/Select/ComboPopup";
const ROW_HEIGHT = 180;
type AuthStatus = "pending" | "authenticated" | "failed";
type SyncStatus = "pending" | "running" | "completed" | "failed";

const createEmptyAccount = (): ITaxAccount => ({
    tax_code: "",
    username: "",
    password_key: "",
    start_date: new Date().toLocaleDateString("sv-SE"),
    enabled: true,
    invoice_type: "[1,2,3,4]",
    auto_sync: false,
    access_code: "",
    type_app: "vaOnline",
    user_app: "",
    pass_app_key: "",
    org_unit: "",

    auth_status: "pending",

    last_login_at: null,
    last_auth_error: null,

    last_sync_at: null,
    last_sync_status: "pending",
    last_sync_error: null,

    created_at: null,
    updated_at: null,

    note: null,
    url_api: null,
    auth_fail_count: 0,
    auth_blocked: false,
    app_auth_status: "",
    app_auth_fail_count: 0,
    app_auth_blocked: false,
    last_app_login_at: null,
    last_app_auth_error: null
});
const authLabels: Record<AuthStatus, string> = {
    pending: "Chưa xác thực",
    authenticated: "Đã xác thực",
    failed: "Xác thực lỗi",
};

const syncLabels: Record<SyncStatus, string> = {
    pending: "Chờ đồng bộ",
    running: "Đang đồng bộ",
    completed: "Hoàn thành",
    failed: "Đồng bộ lỗi",
};

const labelClass = "mb-1.5 block text-xs font-medium text-foreground";

function Field({
    label,
    required,
    children,
}: {
    label: string;
    required?: boolean;
    children: React.ReactNode;
}) {
    return (
        <div className="min-w-0">
            <label className={labelClass}>
                {label}
                {required && <span className="ml-1 text-[#e11c2a]">*</span>}
            </label>
            {children}
        </div>
    );
}

function StatusBadge({
    label,
    variant = "neutral",
}: {
    label: string;
    variant?: "success" | "warning" | "danger" | "neutral" | "info";
}) {
    const styles = {
        success: "bg-green-50 text-green-700 ring-green-600/20",
        warning: "bg-amber-50 text-amber-700 ring-amber-600/20",
        danger: "bg-red-50 text-red-700 ring-red-600/20",
        neutral: "bg-muted text-muted-foreground ring-gray-600/20",
        info: "bg-blue-50 text-blue-700 ring-blue-600/20",
    };

    return (
        <span
            className={`inline-flex items-center rounded px-1.5 py-0.5
                text-[11px] font-medium ring-1 ring-inset ${styles[variant]}`}
        >
            {label}
        </span>
    );
}

function authVariant(status: AuthStatus) {
    if (status === "authenticated") return "success";
    if (status === "failed") return "danger";
    return "warning";
}

interface ITaxAccountManager {
    isRunning: boolean
}
export default function TaxAccountManager({ isRunning }: ITaxAccountManager) {

    const { taxCode } = useParams<{ taxCode: string | undefined }>();
    const {
        accounts,
        saving,
        error,
        createAccount,
        updateAccount,
        updateAutoSync,
        deleteAccount,
    } = useTaxAccounts();

    const [loading, setLoading] = useState(false);

    const { mutateAsync: checkLogin, isPending: isCheckingLogin } = useCheckTaxAccountLogin();

    const { mutateAsync: checkLoginApp, isPending: isCheckingLoginApp } = useCheckLoginApp();

    const [form, setForm] = useState<ITaxAccount>(createEmptyAccount);
    const [selectedTaxCode, setSelectedTaxCode] = useState<string | null>(null);
    const [search, setSearch] = useState("");
    const [isError, setIsError] = useState(false);
    const [isEdit, setIsEdit] = useState(false);
    const [localError, setLocalError] = useState("");

    const filteredAccounts = useMemo(() => {
        const keyword = search.trim().toLowerCase();

        return accounts.filter((item) => {
            const matchesKeyword = [
                item.tax_code,
                item.username,
                item.user_app ?? "",
                item.org_unit ?? "",
            ].some((value) => value.toLowerCase().includes(keyword));

            const matchesError =
                !isError ||
                item.auth_blocked ||
                item.app_auth_blocked;

            return matchesKeyword && matchesError;
        });
    }, [accounts, search, isError]);

    const updateField = <K extends keyof ITaxAccount>(
        key: K,
        value: ITaxAccount[K],
    ) => {
        setForm((prev) => ({ ...prev, [key]: value }));
        setLocalError("");
    };

    const refSecretKeys = useRef<{
        taxPasswordKey: string;
        appPasswordKey: string | null;
    } | null>(null);

    const handleNew = () => {
        setForm(createEmptyAccount());
        setSelectedTaxCode(null);
        refSecretKeys.current = null;
        setIsEdit(false);
        setLocalError("");
    };

    const handleSelect = (account?: ITaxAccount) => {
        if (!account) {
            handleNew();
            return;
        }
        setForm({
            ...account,
            password_key: "",
            pass_app_key: "",
        });
        console.log('org_unit account>>', account.org_unit);
        refSecretKeys.current = {
            taxPasswordKey: account.password_key,
            appPasswordKey: account.pass_app_key,
        };

        setSelectedTaxCode(account.tax_code);
        setIsEdit(true);
        setLocalError("");
    };

    const refTaxCode = useLatest(selectedTaxCode);
    useEffect(() => {
        const accountSelected = filteredAccounts.find(acc => acc.tax_code === refTaxCode.current) ?? filteredAccounts[0];
        handleSelect(accountSelected);
    }, [filteredAccounts])

    useEffect(() => {
        const displayError = localError || error;
        if (displayError) dialog.info(
            displayError,
            "Thông tin lỗi",
        );
    }, [localError, error]);
    const [orgUnits, setOrgUnits] = useState<{ id: string, value: string, tax_code: string }[]>([]);

    useEffect(() => {
        setOrgUnits([]);
    }, [
        form.tax_code,
        form.access_code
    ])

    const handleCheckLoginApp = async () => {
        const password = refSecretKeys.current?.appPasswordKey || form.pass_app_key;
        if (isEmpty(form.access_code) || isEmpty(form.user_app) || isEmpty(password)) {
            showNotification({
                type: "warning",
                message: "Bạn cần nhập đủ thông tin đăng nhập"
            });
            return;
        }
        const taxCode = form.tax_code.trim();
        const nextAccount: ITaxAccount = {
            ...form,
            tax_code: taxCode,
            username: taxCode,
        };
        const checkData = {
            account: {
                ...nextAccount,
                password_key: refSecretKeys.current?.taxPasswordKey ?? "",
                pass_app_key: refSecretKeys.current?.appPasswordKey ?? null
            },
            passwords: {
                taxPassword: form.password_key,
                appPassword: form.pass_app_key
            },
        };
        setLocalError("");
        try {
            const response = await checkLoginApp(checkData);
            setOrgUnits(response.org_units || []);
        } catch (error) {
            setLocalError(
                error instanceof Error ? error.message : String(error),
            );
        }
    }
    const [loadingUpdate, setLoadingUpdate] = useState<{ enabled: boolean, autoSync: boolean }>({ enabled: false, autoSync: false });

    const handleSaveAutoSync = async (fieldChange: 'enabled' | 'autoSync',
        taxCode: string, enabled: boolean,
        autoSync: boolean, accessCode?: string) => {
        if (isRunning) {
            showNotification({
                type: "warning",
                message: "Bạn phải dừng dịch vụ đồng bộ mới được cập nhật!"
            })
            return;
        }
        if (isEmpty(accessCode) && autoSync && fieldChange === "autoSync") {
            showNotification({
                type: "warning",
                message: "Dữ liệu không đủ thông tin để bật cơ chế dồng bộ tự động!"
            })
            return
        };
        setLocalError("");
        try {
            setLoadingUpdate(prev => ({ ...prev, [fieldChange]: true }));
            await updateAutoSync(taxCode, enabled, autoSync);
        } catch (error) {
            setLocalError(
                error instanceof Error ? error.message : String(error),
            );
        } finally {
            setLoadingUpdate(prev => ({ ...prev, [fieldChange]: false }));
        }
    }

    const handleSave = async () => {
        if (isRunning && isEdit) {
            showNotification({
                type: "warning",
                message: "Bạn phải dừng dịch vụ đồng bộ mới được cập nhật!"
            })
            return;
        }
        setLocalError("");
        const taxCode = form.tax_code.trim();

        if (!taxCode || !form.start_date) {
            setLocalError(
                "Vui lòng nhập mã số thuế và ngày bắt đầu.",
            );
            return;
        }

        if (!/^\d{10}(\d{3})?$/.test(taxCode)) {
            setLocalError("Mã số thuế phải có 10 hoặc 13 chữ số.");
            return;
        }

        if (!form.password_key.trim() && !isEdit) {
            setLocalError("Vui lòng nhập khóa tham chiếu mật khẩu thuế.");
            return;
        }

        if (
            form.auto_sync &&
            (
                !form.access_code?.trim() ||
                !form.user_app?.trim() ||
                !form.org_unit?.trim() || (!form.pass_app_key?.trim() && !isEdit)
            )
        ) {
            setLocalError(
                "Khi bật tự động đồng bộ, vui lòng nhập mã truy cập, " +
                "tài khoản ứng dụng và mã đơn vị.",
            );
            return;
        }

        const duplicate = accounts.some(
            (item) =>
                item.tax_code === taxCode &&
                item.tax_code !== selectedTaxCode,
        );

        if (duplicate) {
            setLocalError("Mã số thuế này đã tồn tại.");
            return;
        }

        const nextAccount: ITaxAccount = {
            ...form,
            tax_code: taxCode,
            username: taxCode,
        };

        // Nếu sửa và không nhập mật khẩu mới, giữ key cũ.
        // Nếu có mật khẩu mới, backend sẽ tạo/cập nhật key tương ứng.

        const input: ITaxAccountInput = {
            tax_code: nextAccount.tax_code,
            username: nextAccount.username,
            password_key: nextAccount.password_key,
            start_date: nextAccount.start_date,
            enabled: nextAccount.enabled,
            invoice_type: nextAccount.invoice_type,
            auto_sync: nextAccount.auto_sync,
            access_code: nextAccount.access_code,
            type_app: nextAccount.type_app,
            user_app: nextAccount.user_app,
            pass_app_key: nextAccount.pass_app_key,
            org_unit: nextAccount.org_unit,
            note: nextAccount.note,
            url_api: nextAccount.url_api,
        };

        try {
            setLoading(true);
            // Chỉ truyền mật khẩu thực sự mới nhập trên form.
            // Không truyền key tham chiếu vào passwords.
            const checkData = {
                account: {
                    ...nextAccount,
                    password_key: refSecretKeys.current?.taxPasswordKey ?? "",
                    pass_app_key: refSecretKeys.current?.appPasswordKey ?? null
                },
                passwords: {
                    taxPassword: form.password_key,
                    appPassword: form.pass_app_key
                },
            };

            const response = await checkLogin(checkData);
            input.note = response.name;

            setLocalError("");

            const savedAccount: ITaxAccount = (isEdit && selectedTaxCode) ? await updateAccount(selectedTaxCode, input) : await createAccount(input);

            // Cập nhật lại key tham chiếu từ dữ liệu đã lưu.
            refSecretKeys.current = {
                taxPasswordKey: savedAccount.password_key,
                appPasswordKey: savedAccount.pass_app_key,
            };

            setSelectedTaxCode(taxCode);

            // Không giữ mật khẩu plaintext trong form sau khi lưu.
            setForm({
                ...savedAccount,
                password_key: "",
                pass_app_key: "",
            });

            showNotification({
                type: "success",
                message: `${isEdit ? "Cập nhật thành công: " : "Thêm mới thành công: "}${taxCode}`
            });

            setIsEdit(true);
        } catch (error) {
            console.error("Lỗi lưu tài khoản:", error);
            setLocalError(
                error instanceof Error ? error.message : String(error),
            );
        } finally {
            setLoading(false);
        }
    };
    const navigate = useNavigate();
    const onHistoryInvoice = (taxCode: string) => {
        navigate(`/history-invoice/${taxCode}`);
    }
    const handleDelete = async (taxCode: string) => {
        if (isRunning) {
            showNotification({
                type: "warning",
                message: "Bạn phải dừng dịch vụ đồng bộ mới được cập nhật!"
            })
            return;
        }
        if (saving) return;

        const confirmed = await dialog.question(
            `Bạn có chắc muốn xóa tài khoản ${taxCode}?`,
            "Xóa tài khoản",
        );

        if (!confirmed) return;
        try {
            await deleteAccount(taxCode);
            handleNew();
        } catch (error) {
            setLocalError(JSON.stringify(error));
        }
    };
    // Ưu tiên lỗi kiểm tra form, sau đó đến lỗi từ hook.


    const containerRef = useRef<HTMLDivElement>(null);

    const {
        visibleData,
        offsetTop,
        offsetBottom,
        startIndex,
    } = useVirtualized({
        data: filteredAccounts,
        rowHeight: ROW_HEIGHT,
        containerRef,
    });

    const tooltipContentError = useCallback((error1: string | null, error2: string | null): ReactNode => {
        return (
            <div className="space-y-2 p-1">
                <div className="font-bold">
                    Thông tin lỗi
                </div>

                <div className="rounded-sm border border-yellow-700/50 bg-yellow-950/40 px-2.5 py-2 text-xs text-yellow-100">
                    <div className="mb-1 font-semibold text-yellow-300">
                        Chi tiết
                    </div>

                    <div className="italic leading-relaxed text-yellow-100/90">
                        {error1 && <i className="block">
                            1. {error1}
                        </i>}

                        {error2 && <i className="block">
                            2.{error2}
                        </i>}
                    </div>
                </div>
            </div>
        );
    }, [])

    const label_access_code = useMemo(() => {
        switch (form.type_app) {
            case "vaOnline":
                return "Domain truy cập";
            case "vaOne":
                return "Mã số thuế";
            case "vaShop":
                return "Mã cửa hàng"
            default:
                return "Mã truy cập";
        }
    }, [form.type_app]);

    return (
        <div className="flex h-full min-h-0 flex-col bg-background text-foreground p-2 w-225" >
            <div
                className="grid min-h-0 flex-1
                grid-cols-2 gap-1
                overflow-hidden"
            >
                {/* LEFT: Form */}
                <section className="flex min-h-0 flex-col border border-gray-300 rounded-md">
                    <div className="flex shrink-0 items-center justify-between
                        border-b border-gray-300 px-4 py-3 h-10">
                        <div className="flex items-center gap-2 text-xs
                                font-semibold uppercase tracking-wide
                                text-muted-foreground">
                            <KeyRound size={14} />
                            Tài khoản cơ quan thuế
                        </div>
                        <button
                            type="button"
                            onClick={handleNew}
                            className="text-sm font-medium text-[#e11c2a]
                                            hover:underline hover:cursor-pointer"
                        >
                            + Thêm tài khoản
                        </button>
                    </div>

                    <div className="min-h-0 flex-1 space-y-1 p-4">
                        {/* Tax credentials */}
                        <div className="space-y-3">
                            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                                <Input
                                    label="Mã số thuế"
                                    disabled={isEdit}
                                    sizeInput="xs"
                                    autoFocus
                                    required
                                    value={form.tax_code}
                                    onChange={(e) =>
                                        updateField("tax_code", e.target.value)
                                    }
                                />

                                <DateInput
                                    label="Ngày bắt đầu đồng bộ"
                                    sizeInput="xs"
                                    required
                                    value={form.start_date ?? ""}
                                    onChange={(value) =>
                                        updateField("start_date", value ?? "")
                                    }
                                />

                                {/* <Input
                                    label="Tên đăng nhập"
                                    disabled={isEdit}
                                    sizeInput="xs"
                                    required
                                    value={form.username}
                                    onChange={(e) =>
                                        updateField("username", e.target.value)
                                    }
                                /> */}

                                <Input
                                    label={<div className="flex gap-1 items-center">
                                        <span>Mật khẩu thuế</span>
                                        <Tooltip title="Nhập để cập nhật mật khẩu" subTitle={isEdit ? "(nếu để trắng thì hệ thống giữ nguyên mật khẩu cũ))" : undefined}> <MessageCircleMore size={12} className="text-blue-500" /></Tooltip>
                                    </div>}
                                    password
                                    // placeholder="Mật khẩu truy cập..."
                                    sizeInput="xs"
                                    required={!isEdit}
                                    value={form.password_key}
                                    onChange={(e) =>
                                        updateField("password_key", e.target.value)
                                    }
                                />
                                <div className="flex items-end justify-end">
                                    <Switch
                                        label="Kích hoạt"
                                        labelClass="text-xs"
                                        // description="Cho phép worker xử lý tài khoản này"
                                        checked={form.enabled}
                                        onChange={(value) =>
                                            updateField("enabled", value)
                                        }
                                    />
                                </div>
                            </div>
                            <Input
                                label="Công ty"
                                disabled={true}
                                sizeInput="xs"
                                value={form.note ?? ""}
                                onChange={(e) =>
                                    updateField("note", e.target.value)
                                }
                            />
                            <InvoiceTypeCheckbox
                                value={form.invoice_type}
                                onChange={(value) => updateField("invoice_type", value)}
                            />
                        </div>

                        <div className="border-t border-gray-300" />
                        <Switch
                            label="Tự động đồng bộ"
                            labelClass="text-xs"
                            description="Cho phép worker xử lý đồng bộ"
                            checked={form.auto_sync}
                            onChange={(value) =>
                                updateField("auto_sync", value)
                            }
                        />
                        {/* Target app */}
                        {form.auto_sync && <div className="space-y-3">

                            <Field label="Ứng dụng đích" required>
                                <div className="grid grid-cols-3 border border-gray-300 rounded">
                                    {(["vaOnline", "vaOne", "vaShop"] as const).map((app, idx) => (
                                        <button
                                            key={app}
                                            type="button"
                                            onClick={() => {
                                                if (app === "vaShop") updateField("org_unit", "VP");
                                                updateField("type_app", app);
                                            }}
                                            className={`flex h-7 items-center justify-center
                                                gap-2 text-sm font-medium
                                                transition-colors ${form.type_app === app
                                                    ? "border-[#e11c2a] bg-[#e11c2a]/5 text-[#e11c2a]"
                                                    : "hover:bg-muted"
                                                } ${idx !== 2 ? ' border-r border-gray-300' : ''}`}
                                        >
                                            {form.type_app === app && <Check size={15} />}
                                            {app}
                                        </button>
                                    ))}
                                </div>
                            </Field>

                            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                                <Input
                                    label={label_access_code}
                                    sizeInput="xs"
                                    required
                                    value={form.access_code}
                                    onChange={(e) =>
                                        updateField("access_code", e.target.value)
                                    }
                                />

                                <Input
                                    label="Tài khoản ứng dụng"
                                    sizeInput="xs"
                                    required
                                    value={form.user_app ?? ""}
                                    onChange={(e) =>
                                        updateField("user_app", e.target.value)
                                    }
                                />

                                <Input
                                    label={<div className="flex gap-1 items-center">
                                        <span>Mật khẩu ứng dụng</span>
                                        <Tooltip title="Nhập để cập nhật mật khẩu" subTitle={isEdit ? "(nếu để trắng thì hệ thống giữ nguyên mật khẩu cũ))" : undefined}> <MessageCircleMore size={12} className="text-blue-500" /></Tooltip>
                                    </div>}
                                    password
                                    sizeInput="xs"
                                    value={form.pass_app_key ?? ""}
                                    // placeholder="Mật khẩu áp dụng..."
                                    onChange={(e) =>
                                        updateField("pass_app_key", e.target.value)
                                    }
                                />


                                {["vaOnline", "vaOne"].includes(form.type_app) ?
                                    <div className="flex-1 flex flex-col gap-1">
                                        <span className="text-xs font-medium">Đơn vị dữ liệu</span>
                                        <div className="flex gap-1 items-center">
                                            <ComboPopup
                                                value={orgUnits.find(i => i.id === form.org_unit)}
                                                data={orgUnits}
                                                placement="bottom-start"
                                                className="w-45 h-7"
                                                onClear={() => {
                                                    updateField("org_unit", "")
                                                }}
                                                onSelect={(item) => {
                                                    updateField("org_unit", item.id)
                                                }}
                                                renderDisplay={(item) => <span className="text-xs px-2">{form.org_unit}</span>}
                                                renderItem={(item) => <div key={item?.id} className="text-xs"><span className="text-blue-500">{item?.tax_code}</span> - {item?.value}</div>}
                                            />
                                            <Tooltip title="Lấy danh sách đơn vị"><Button variant="secondary" icon={
                                                <RefreshCw size={15} className={isCheckingLoginApp ? "animate-spin" : undefined} />
                                            } disabled={isCheckingLoginApp} iconOnly={true} onClick={handleCheckLoginApp} /></Tooltip>
                                        </div>
                                    </div> : <div />}
                            </div>

                            <Input
                                label="Đường dẫn truy cập (cài riêng mới phải nhập)"
                                placeholder={"ví dụ: http://192.168.1.123:9999"}
                                sizeInput="xs"
                                value={form.url_api ?? ""}
                                onChange={(e) =>
                                    updateField("url_api", e.target.value)
                                }
                            />
                        </div>}

                        {/* Error */}
                        {/* {displayError && (
                            <div className="flex items-start gap-2 rounded border
                                border-red-200 bg-red-50 p-3 text-sm text-red-700">
                                <CircleAlert size={16} className="mt-0.5 shrink-0" />
                                <span>{displayError}</span>
                            </div>
                        )} */}
                    </div>

                    {/* Form actions */}
                    {loading ? <div className="flex justify-center py-2"><Loading /></div> : <footer className="flex shrink-0 flex-wrap justify-end items-center gap-2
                        border-t border-gray-300 bg-muted/20 p-3">
                        {/* <Button icon={<RefreshCw size={15} />} onClick={handleNew} className="rounded-full" variant="secondary">
                            Làm mới
                        </Button> */}
                        <Button icon={isEdit ? <Save size={15} /> : <Plus size={15} />} onClick={handleSave} className="rounded-full">
                            {isEdit ? "Lưu thay đổi" : "Ghi mới tài khoản"}
                        </Button>
                    </footer>}
                </section>

                {/* RIGHT: List */}
                <section className="flex min-h-0 flex-col border border-gray-300 rounded-md">
                    <div className="flex shrink-0 items-center justify-between
                        border-b border-gray-300 px-4 py-3 h-10">
                        <div className="flex items-center gap-2 text-xs
                                font-semibold tracking-wide
                                text-muted-foreground">
                            <List size={14} />
                            Danh sách tài khoản
                        </div>

                        <div className="w-45">
                            <Input
                                placeholder="Tìm mã số thuế..."
                                sizeInput="xs"
                                required
                                value={search}
                                onChange={(e) => setSearch(e.target.value)}
                            />
                        </div>
                        <Switch
                            label="Lỗi"
                            labelClass="text-xs"
                            checked={isError}
                            onChange={(value) =>
                                setIsError(value)
                            }
                        />
                    </div>
                    <div className="min-h-0 flex-1 overflow-y-auto p-3 bg-gray-100">
                        {visibleData.length === 0 ? (
                            <div className="flex h-full min-h-48 flex-col
                                items-center justify-center gap-3 text-center">
                                <div className="flex size-12 items-center
                                    justify-center rounded-full bg-muted">
                                    <Building2
                                        size={22}
                                        className="text-muted-foreground"
                                    />
                                </div>
                                <div>
                                    <p className="text-sm font-medium">
                                        {search
                                            ? "Không tìm thấy tài khoản"
                                            : "Chưa có tài khoản thuế"}
                                    </p>
                                    <p className="mt-1 text-xs text-muted-foreground">
                                        {search
                                            ? "Thử từ khóa tìm kiếm khác."
                                            : "Thêm tài khoản để bắt đầu cấu hình."}
                                    </p>
                                </div>
                                {!search && (
                                    <button
                                        type="button"
                                        onClick={handleNew}
                                        className="text-sm font-medium text-[#e11c2a]
                                            hover:underline hover:cursor-pointer"
                                    >
                                        + Thêm tài khoản đầu tiên
                                    </button>
                                )}
                            </div>
                        ) : (
                            <div className="space-y-2" ref={containerRef}>
                                <div style={{ height: offsetTop }} />
                                {visibleData.map((account, index) => {
                                    const selected =
                                        selectedTaxCode === account.tax_code;
                                    return (
                                        <div
                                            key={account.tax_code}
                                            onClick={() => handleSelect(account)}
                                            className={`w-full rounded border border-gray-300 p-3 text-left
                                                transition-colors ${selected
                                                    ? "border-[#e11c2a] bg-[#e11c2a]/[0.035]"
                                                    : "hover:border-border hover:bg-muted/40 bg-white"
                                                }`}
                                            style={{ height: ROW_HEIGHT }}
                                        >
                                            <div className="flex items-start gap-3">
                                                <div className={`flex flex-col size-10 shrink-0
                                                    items-center justify-center rounded
                                                    ${selected
                                                        ? "bg-[#e11c2a]/10 text-[#e11c2a]"
                                                        : "bg-muted text-muted-foreground"
                                                    }`}>
                                                    <span className="text-xs">{index + 1}</span>
                                                    <Building2 size={18} />
                                                </div>
                                                <div className="min-w-0 flex-1">
                                                    <div className="flex items-start
                                                        justify-between gap-2">
                                                        <div className="min-w-0 flex-1">
                                                            <div className="flex gap-1 items-center">
                                                                <p className="truncate
                                                                text-sm font-semibold">
                                                                    {account.tax_code}
                                                                </p>
                                                                {loadingUpdate.enabled ? <Loader className="animate-spin" size={15} /> : <Switch
                                                                    labelClass="text-xs"
                                                                    checked={account.enabled}
                                                                    onChange={(enabled) => handleSaveAutoSync("enabled", account.tax_code, enabled, account.auto_sync, account.access_code)}
                                                                />}
                                                                {(account.last_auth_error || account.last_app_auth_error) && <Tooltip
                                                                    content={tooltipContentError(account.last_auth_error, account.last_app_auth_error)}
                                                                >
                                                                    <div className="flex gap-1 justify-center text-[11px] text-orange-500 items-center">
                                                                        <span>Thông tin lỗi</span>
                                                                        <MessageCircleWarning size={18} />
                                                                    </div>
                                                                </Tooltip>}
                                                            </div>
                                                            <p className="mt-1 truncate
                                                                text-xs text-muted-foreground">
                                                                {account.invoice_type}
                                                            </p>
                                                        </div>
                                                        <Tooltip title="Xóa mã số thuế"
                                                            className="px-1 pb-1 bg-amber-50 rounded-full hover:cursor-pointer border border-gray-300"
                                                            onClick={(e) => {
                                                                e.stopPropagation();
                                                                handleDelete(account.tax_code);
                                                            }}
                                                        >
                                                            <Trash2
                                                                size={14}
                                                                className="mt-1 shrink-0 text-red-600
                                                                text-muted-foreground"
                                                            />
                                                        </Tooltip>
                                                        <Tooltip title="Xem lịch sử truyền nhận dữ liệu"
                                                            className="px-1 pb-1 bg-amber-50 rounded-full hover:cursor-pointer border border-gray-300"
                                                            onClick={(e) => {
                                                                e.stopPropagation();
                                                                onHistoryInvoice(account.tax_code);
                                                            }}
                                                        >
                                                            <History
                                                                size={14}
                                                                className="mt-1 shrink-0 text-purple-600
                                                                text-muted-foreground"
                                                            />
                                                        </Tooltip>
                                                    </div>
                                                    <span className="text-xs truncate">{account.note}</span>
                                                    <div className="mt-3 flex flex-wrap
                                                        items-center gap-1.5">
                                                        <StatusBadge
                                                            label={authLabels[account.auth_status]}
                                                            variant={authVariant(account.auth_status)}
                                                        />
                                                        {account.auto_sync && <StatusBadge
                                                            label={account.type_app}
                                                            variant="neutral"
                                                        />}
                                                        <StatusBadge
                                                            label={
                                                                account.enabled
                                                                    ? "Đang kích hoạt"
                                                                    : "Đã tắt"
                                                            }
                                                            variant={
                                                                account.enabled
                                                                    ? "success"
                                                                    : "neutral"
                                                            }
                                                        />
                                                    </div>

                                                    <div className="mt-3 flex flex-wrap
                                                        items-center justify-between gap-2
                                                        border-t border-gray-300 pt-2.5">
                                                        <div className="flex items-center
                                                            gap-1.5 text-xs
                                                            text-muted-foreground">
                                                            <RefreshCw size={12} />
                                                            {syncLabels[account.last_sync_status]}
                                                        </div>
                                                        {loadingUpdate.autoSync ? <Loader className="animate-spin" size={15} /> : <Switch
                                                            label="Tự động"
                                                            labelClass="text-xs"
                                                            checked={account.auto_sync}
                                                            onChange={(autoSync) => handleSaveAutoSync("autoSync", account.tax_code, account.enabled, autoSync, account.access_code)}
                                                        />}
                                                    </div>
                                                    <div className="mt-2 flex items-center
                                                        gap-1.5 text-[11px]
                                                        text-muted-foreground">
                                                        <Clock size={12} />
                                                        Ngày bắt đầu: {formatDate(account.start_date)}
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    );
                                })}
                                <div style={{ height: offsetBottom }} />
                            </div>
                        )}
                    </div>
                </section>
            </div>
        </div>
    );
}
