import { useMutation, useQuery } from "@tanstack/react-query";
import { IGetSyncDaysParams, IGetSyncInvoicesParams, ISyncConfig, syncInvoiceService } from "../service/syncInvoicesService";

export const useGetInvoiceStats = (taxCode: string) =>
    useQuery({
        queryKey: ["invoice-stats", taxCode],
        queryFn: () => syncInvoiceService.getInvoiceStats(taxCode),
        enabled: !!taxCode,
        staleTime: 0,
        refetchOnMount: "always",
    });

export const useGetSyncDays = (
    taxCode: string,
    isSuccess?: boolean
) =>
    useQuery({
        queryKey: ["sync-days", taxCode, isSuccess],
        queryFn: () => syncInvoiceService.getSyncDays(taxCode, isSuccess),
        enabled: !!taxCode,
        staleTime: 0,
        refetchOnMount: "always",
    });

export const useGetSyncInvoices = (
    params: IGetSyncInvoicesParams
) =>
    useQuery({
        queryKey: ["sync-invoices", params],
        queryFn: () =>
            syncInvoiceService.getSyncInvoices(
                params.taxCode,
                params.filters,
                params.page,
                params.page_size
            ),
        enabled: !!params.taxCode,
        staleTime: 0,
        refetchOnMount: "always",
    });


export const useResetSyncStatus = () =>
    useMutation({
        mutationFn: (param: ISyncConfig) =>
            syncInvoiceService.resetSyncStatus(param),
    });