
import { invoke } from "@tauri-apps/api/core";

export type TypeApp = "vaOnline" | "vaOne" | "vaShop";

export type AuthStatus =
    | "pending"
    | "authenticated"
    | "failed";

export type SyncStatus =
    | "pending"
    | "running"
    | "completed"
    | "failed";

export interface ITaxAccount {
    tax_code: string;
    username: string;
    password_key: string;
    start_date: string;
    enabled: boolean;
    invoice_type: string;
    auto_sync: boolean;
    access_code: string;
    type_app: TypeApp;
    user_app: string | null;
    pass_app_key: string | null;
    org_unit: string | null;

    auth_status: AuthStatus;
    last_login_at: string | null;
    last_auth_error: string | null;

    last_sync_at: string | null;
    last_sync_status: SyncStatus;
    last_sync_error: string | null;

    created_at: string | null;
    updated_at: string | null;

    note: string | null;
    url_api: string | null;
    auth_fail_count: number;
    auth_blocked: boolean;
    app_auth_status: string;
    app_auth_fail_count: number;
    app_auth_blocked: boolean;
    last_app_login_at: string | null;
    last_app_auth_error: string | null;
}

export type ITaxAccountInput = Pick<
    ITaxAccount,
    | "tax_code"
    | "username"
    | "password_key"
    | "start_date"
    | "enabled"
    | "invoice_type"
    | "auto_sync"
    | "access_code"
    | "type_app"
    | "user_app"
    | "pass_app_key"
    | "org_unit"
    | "note"
    | "url_api"
>;

interface LoginResponse {
    token: string,
    expires_in: number | null,
    name: string,
}


interface LoginAppResponse {
    org_units: any[]
}

export const taxAccountService = {
    getAll: () =>
        invoke<ITaxAccount[]>("get_tax_accounts"),

    create: (input: ITaxAccountInput): Promise<ITaxAccount> =>
        invoke<ITaxAccount>("create_tax_account", { input }),

    update: (
        taxCode: string,
        input: ITaxAccountInput,
    ): Promise<ITaxAccount> =>
        invoke<ITaxAccount>("update_tax_account", {
            taxCode,
            input,
        }),

    updateAutoSync: (
        taxCode: string,
        enabled: boolean,
        autoSync: boolean,
    ): Promise<ITaxAccount> =>
        invoke<ITaxAccount>("update_auto_sync", {
            taxCode,
            enabled,
            autoSync,
        }),

    delete: (taxCode: string) =>
        invoke<void>("delete_tax_account", {
            taxCode,
        }),

    checkLogin: (
        account: ITaxAccount,
        passwords?: {
            taxPassword?: string;
            appPassword?: string | null;
        },
    ): Promise<LoginResponse> =>
        invoke<LoginResponse>("check_tax_application_login", {
            input: {
                account,
                taxPassword: passwords?.taxPassword,
                appPassword: passwords?.appPassword,
            },
        }),

    checkLoginApp: (
        account: ITaxAccount,
        passwords?: {
            taxPassword?: string;
            appPassword?: string | null;
        },
    ): Promise<LoginAppResponse> =>
        invoke<LoginAppResponse>("check_app_login", {
            input: {
                account,
                taxPassword: passwords?.taxPassword,
                appPassword: passwords?.appPassword,
            },
        }),
};
