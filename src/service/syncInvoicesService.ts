import { invoke } from "@tauri-apps/api/core";

export type IInvoiceStatsGroup = {
    invoice_type: number;
    detail_status: string;
    upload_status: string;
    count: number;
};

export type IInvoiceStats = {
    tax_code: string;
    total: number;
    groups: IInvoiceStatsGroup[];
};

export type InvoiceStatus =
    | "pending"
    | "running"
    | "completed"
    | "failed";

export type ISyncDay = {
    id: number;
    tax_code: string;
    invoice_type: number;
    sync_date: string;
    list_status: string;
    status: string;
    last_error: string | null;
    attempt_count: number;
    started_at: string | null;
    completed_at: string | null;
    updated_at: string;
};

export interface ISyncInvoice {
    tax_code: string;
    invoice_type: number;
    invoice_key: string;

    dvtte: string | null;
    tgia: number | null;
    nbmst: string | null;
    nbnmmst: string | null;
    nbnmten: string | null;
    khhdon: string | null;
    shdon: number | null;
    khmshdon: number | null;
    invoice_date: string;

    tgtcthue: number | null;
    tgtthue: number | null;
    tgtttbso: number | null;
    tthai: number | null;
    ttxly: number | null;

    detail_status: InvoiceStatus;
    upload_status: InvoiceStatus;

    detail_attempt_count: number;
    upload_attempt_count: number;

    detail_error: string | null;
    upload_error: string | null;

    updated_at: string;
    completed_at: string | null;
}

export interface ISyncInvoiceFilters {
    tax_code?: string;
    nbmst?: string;
    khhdon?: string;
    shdon?: number;
    invoice_type?: number;
    from_date?: string;
    to_date?: string;
    detail_status?: InvoiceStatus;
    upload_status?: InvoiceStatus;
}

export interface ISyncInvoiceParams extends ISyncInvoiceFilters {
    page: number;
    page_size: number;
}

export interface IPaginatedResult<T> {
    items: T[];
    total: number;
    page: number;
    page_size: number;
    total_pages: number;
}

export interface IGetSyncInvoicesParams {
    taxCode: string;
    filters: ISyncInvoiceFilters;
    page: number;
    page_size: number;
}

export interface IGetSyncDaysParams {
    taxCode: string;
    isSuccess?: boolean
}

export interface ISyncConfig {
    taxCode: string,
    fromDate?: string,
    toDate?: string,
    taxSync: boolean,
    appSync: boolean,
}

export const syncInvoiceService = {
    getInvoiceStats: (taxCode: string) =>
        invoke<IInvoiceStats>("get_invoice_stats", { taxCode }),

    getSyncDays: (taxCode: string, isSuccess?: boolean) =>
        invoke<ISyncDay[]>("get_sync_days", { taxCode, isSuccess }),

    getSyncInvoices: (
        taxCode: string,
        filters: ISyncInvoiceFilters,
        page: number,
        page_size: number,
    ) =>
        invoke<IPaginatedResult<ISyncInvoice>>("get_sync_invoices", {
            taxCode,
            filters,
            page,
            pageSize: page_size,
        }),

    resetSyncStatus: (param: ISyncConfig): Promise<void> =>
        invoke<void>("reset_sync_status", {
            taxCode: param.taxCode,
            fromDate: param.fromDate,
            toDate: param.toDate,
            taxSync: param.taxSync,
            appSync: param.appSync,
        }),
}