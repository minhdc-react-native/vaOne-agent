
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
    taxAccountService,
    type ITaxAccount,
    type ITaxAccountInput,
} from "../service/taxAccountService";

export const TAX_ACCOUNTS_QUERY_KEY = ["taxAccounts"] as const;

const getErrorMessage = (err: unknown): string =>
    err instanceof Error ? err.message : String(err);

export function useTaxAccounts() {
    const queryClient = useQueryClient();

    // 1. Tải danh sách tài khoản
    const accountsQuery = useQuery({
        queryKey: TAX_ACCOUNTS_QUERY_KEY,
        queryFn: () => taxAccountService.getAll(),
    });

    // Hàm refresh thủ công khi cần
    const loadAccounts = async () => {
        await accountsQuery.refetch({ throwOnError: true });
    };

    // 2. Thêm tài khoản
    const createMutation = useMutation({
        mutationFn: (input: ITaxAccountInput): Promise<ITaxAccount> =>
            taxAccountService.create(input),

        onSuccess: () =>
            queryClient.invalidateQueries({
                queryKey: TAX_ACCOUNTS_QUERY_KEY,
            }),
    });

    // 3. Sửa tài khoản
    const updateMutation = useMutation({
        mutationFn: ({
            taxCode,
            input,
        }: {
            taxCode: string;
            input: ITaxAccountInput;
        }): Promise<ITaxAccount> =>
            taxAccountService.update(taxCode, input),

        onSuccess: () =>
            queryClient.invalidateQueries({
                queryKey: TAX_ACCOUNTS_QUERY_KEY,
            }),
    });

    const saveAutoSync = useMutation({
        mutationFn: ({
            taxCode,
            enabled,
            autoSync,
        }: {
            taxCode: string;
            enabled: boolean;
            autoSync: boolean;
        }): Promise<ITaxAccount> =>
            taxAccountService.updateAutoSync(taxCode, enabled, autoSync),

        onSuccess: () =>
            queryClient.invalidateQueries({
                queryKey: TAX_ACCOUNTS_QUERY_KEY,
            }),
    });

    // 4. Xóa tài khoản
    const deleteMutation = useMutation({
        mutationFn: (taxCode: string) =>
            taxAccountService.delete(taxCode),
        onSuccess: () =>
            queryClient.invalidateQueries({
                queryKey: TAX_ACCOUNTS_QUERY_KEY,
            }),
    });

    // Lấy lỗi gần nhất từ query hoặc mutation
    const error =
        accountsQuery.error ??
        createMutation.error ??
        updateMutation.error ??
        deleteMutation.error;

    return {
        accounts: accountsQuery.data ?? [],
        saving:
            createMutation.isPending ||
            updateMutation.isPending ||
            deleteMutation.isPending,

        error: error ? getErrorMessage(error) : null,

        loadAccounts,

        createAccount: createMutation.mutateAsync,

        updateAccount: (
            taxCode: string,
            input: ITaxAccountInput,
        ) => updateMutation.mutateAsync({ taxCode, input }),

        updateAutoSync: (
            taxCode: string,
            enabled: boolean,
            autoSync: boolean,
        ) => saveAutoSync.mutateAsync({ taxCode, enabled, autoSync }),

        deleteAccount: deleteMutation.mutateAsync,
    };
}

// Kiểm tra đăng nhập: không làm thay đổi danh sách tài khoản
interface CheckTaxAccountLoginParams {
    account: ITaxAccount;
    passwords?: {
        taxPassword?: string;
        appPassword?: string | null;
    };
}

export const useCheckTaxAccountLogin = () =>
    useMutation({
        mutationFn: ({
            account,
            passwords,
        }: CheckTaxAccountLoginParams) =>
            taxAccountService.checkLogin(account, passwords),
    });

export const useCheckLoginApp = () =>
    useMutation({
        mutationFn: ({
            account,
            passwords,
        }: CheckTaxAccountLoginParams) =>
            taxAccountService.checkLoginApp(account, passwords),
    });