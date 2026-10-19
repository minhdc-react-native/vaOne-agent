
import { useCallback, useMemo, useRef, useState } from "react";
import {
    Building2,
    Check,
    CircleAlert,
    Clock,
    History,
    KeyRound,
    List,
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
import { useCheckTaxAccountLogin, useTaxAccounts } from "../hook/useTaxAccounts";
import { ITaxAccount, ITaxAccountInput } from "../service/taxAccountService";
import { Loading } from "../components/Loading";
import InvoiceTypeCheckbox from "./InvoiceTypeCheckbox";
import { Tooltip } from "../components/Tooltip/Tooltip";
import { showNotification } from "../lib/notification";
import { useNavigate } from "react-router-dom";

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

function syncVariant(status: SyncStatus) {
    if (status === "completed") return "success";
    if (status === "failed") return "danger";
    if (status === "running") return "info";
    return "neutral";
}

export default function TaxAccountManager() {
    const {
        accounts,
        saving,
        error,
        createAccount,
        updateAccount,
        deleteAccount,
    } = useTaxAccounts();

    const [loading, setLoading] = useState(false);

    const { mutateAsync: checkLogin, isPending: isCheckingLogin } = useCheckTaxAccountLogin();

    const [form, setForm] = useState<ITaxAccount>(createEmptyAccount);
    const [selectedTaxCode, setSelectedTaxCode] = useState<string | null>(null);
    const [search, setSearch] = useState("");
    const [isEdit, setIsEdit] = useState(false);
    const [localError, setLocalError] = useState("");

    const filteredAccounts = useMemo(() => {
        const keyword = search.trim().toLowerCase();

        return accounts.filter((item) =>
            [
                item.tax_code,
                item.username,
                item.user_app ?? "",
                item.org_unit ?? "",
            ].some((value) => value.toLowerCase().includes(keyword)),
        );
    }, [accounts, search]);

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

    const handleSelect = (account: ITaxAccount) => {
        setForm({
            ...account,
            password_key: "",
            pass_app_key: "",
        });

        refSecretKeys.current = {
            taxPasswordKey: account.password_key,
            appPasswordKey: account.pass_app_key,
        };

        setSelectedTaxCode(account.tax_code);
        setIsEdit(true);
        setLocalError("");
    };

    const handleSave = async () => {

        const taxCode = form.tax_code.trim();
        console.log('form>>', form);
        if (!taxCode || !form.start_date) {
            setLocalError(
                "Vui lòng nhập mã số thuế và ngày bắt đầu. abc",
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

            await checkLogin(checkData);

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
    const displayError = localError || error;

    return (
        <div className="flex h-full min-h-0 flex-col bg-background text-foreground p-2 w-200" >
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

                    <div className="min-h-0 flex-1 space-y-1 overflow-y-auto p-4">
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
                                    label="Ngày bắt đầu"
                                    disabled={isEdit}
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
                                    label="Mật khẩu thuế"
                                    disabled={isEdit}
                                    password
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
                                        // description="Cho phép worker xử lý tài khoản này"
                                        checked={form.enabled}
                                        onChange={(value) =>
                                            updateField("enabled", value)
                                        }
                                    />
                                </div>
                            </div>
                            <InvoiceTypeCheckbox
                                value={form.invoice_type}
                                onChange={(value) => updateField("invoice_type", value)}
                            />
                        </div>

                        <div className="border-t border-gray-300" />
                        <Switch
                            label="Tự động đồng bộ"
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
                                    label="Mã truy cập"
                                    sizeInput="xs"
                                    required
                                    value={form.access_code}
                                    onChange={(e) =>
                                        updateField("access_code", e.target.value)
                                    }
                                />

                                {["vaOnline", "vaOne"].includes(form.type_app) ? <Input
                                    label="Mã đơn vị"
                                    sizeInput="xs"
                                    value={form.org_unit ?? ""}
                                    onChange={(e) =>
                                        updateField("org_unit", e.target.value)
                                    }
                                /> : <div />}

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
                                    label="Mật khẩu ứng dụng"
                                    password
                                    sizeInput="xs"
                                    value={form.pass_app_key ?? ""}
                                    placeholder="Keychain reference"
                                    onChange={(e) =>
                                        updateField("pass_app_key", e.target.value)
                                    }
                                />
                            </div>
                        </div>}
                        {/* Error */}
                        {displayError && (
                            <div className="flex items-start gap-2 rounded border
                                border-red-200 bg-red-50 p-3 text-sm text-red-700">
                                <CircleAlert size={16} className="mt-0.5 shrink-0" />
                                <span>{displayError}</span>
                            </div>
                        )}
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
                    </div>
                    <div className="min-h-0 flex-1 overflow-y-auto p-3 bg-gray-100">
                        {filteredAccounts.length === 0 ? (
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
                            <div className="space-y-2">
                                {filteredAccounts.map((account, index) => {
                                    const selected =
                                        selectedTaxCode === account.tax_code;

                                    return (
                                        <button
                                            type="button"
                                            key={account.tax_code}
                                            onClick={() => handleSelect(account)}
                                            className={`w-full rounded border border-gray-300 p-3 text-left
                                                transition-colors ${selected
                                                    ? "border-[#e11c2a] bg-[#e11c2a]/[0.035]"
                                                    : "hover:border-border hover:bg-muted/40 bg-white"
                                                }`}
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
                                                            <p className="truncate
                                                                text-sm font-semibold">
                                                                {account.tax_code}
                                                            </p>
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

                                                        <div className="flex items-center gap-2">
                                                            <span className="text-xs
                                                                text-muted-foreground">
                                                                Tự động
                                                            </span>
                                                            <span className={`relative
                                                                inline-flex h-4 w-7
                                                                rounded-full transition-colors
                                                                ${account.auto_sync
                                                                    ? "bg-[#e11c2a]"
                                                                    : "bg-gray-300"
                                                                }`}>
                                                                <span className={`absolute
                                                                    top-0.5 size-3 rounded-full
                                                                    bg-white transition-transform
                                                                    ${account.auto_sync
                                                                        ? "translate-x-3.5"
                                                                        : "translate-x-0.5"
                                                                    }`} />
                                                            </span>
                                                        </div>
                                                    </div>

                                                    <div className="mt-2 flex items-center
                                                        gap-1.5 text-[11px]
                                                        text-muted-foreground">
                                                        <Clock size={12} />
                                                        Ngày bắt đầu: {account.start_date}
                                                    </div>

                                                    {account.last_auth_error && (
                                                        <p className="mt-2 truncate text-xs
                                                            text-red-600">
                                                            {account.last_auth_error}
                                                        </p>
                                                    )}
                                                    {account.last_sync_error && (
                                                        <p className="mt-1 truncate text-xs
                                                            text-red-600">
                                                            {account.last_sync_error}
                                                        </p>
                                                    )}
                                                </div>
                                            </div>
                                        </button>
                                    );
                                })}
                            </div>
                        )}
                    </div>
                </section>
            </div>
        </div>
    );
}
