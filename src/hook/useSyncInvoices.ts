import { useMutation } from "@tanstack/react-query";
import { IGetSyncDaysParams, IGetSyncInvoicesParams, ISyncConfig, syncInvoiceService } from "../service/syncInvoicesService";

export const useGetInvoiceStats = () =>
    useMutation({
        mutationFn: (taxCode: string) =>
            syncInvoiceService.getInvoiceStats(taxCode),
    });

export const useGetSyncDays = () =>
    useMutation({
        mutationFn: ({ taxCode, isSuccess }: IGetSyncDaysParams) =>
            syncInvoiceService.getSyncDays(taxCode, isSuccess),
    });

export const useGetSyncInvoices = () =>
    useMutation({
        mutationFn: ({
            filters,
            page,
            page_size,
        }: IGetSyncInvoicesParams) =>
            syncInvoiceService.getSyncInvoices(filters, page, page_size),
    });


export const useResetSyncStatus = () =>
    useMutation({
        mutationFn: (param: ISyncConfig) =>
            syncInvoiceService.resetSyncStatus(param),
    });