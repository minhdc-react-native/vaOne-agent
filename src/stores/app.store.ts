import { create } from "zustand";
import { persist } from "zustand/middleware";
import { mInvoiceService } from "../api/services/mInvoice.service";
interface ILogin {
    tenantId: string,
    source: string;
    username: string;
    password: string;
    token: string;
    reConnect: boolean;
    taxCode: string;
    idAccount: string;
    info: any
}

interface ISyncProgress {
    completed: number;
    total: number;
    invoice: {
        invoiceDate: string;
        invoiceNumber: number | null;
        invoiceSerial: string;
    } | null;
}

interface IAppState {
    autostartInitialized: boolean,
    setAutostartInitialized: (autostartInitialized: boolean) => void;
    delayRequest: number;
    setDelayRequest: (delayRequest: number) => void;

    savePassword: Record<string, string>;
    setLogin: (login: ILogin | null) => void;
    login: ILogin | null;

    syncProgress: ISyncProgress | null;
    setSyncProgress: (payload: Record<string, any>) => void;
}
export const useAppStore = create<IAppState>()(
    persist(
        (set) => ({
            autostartInitialized: false,
            setAutostartInitialized: (autostartInitialized) => set({ autostartInitialized }),
            delayRequest: 1500,
            setDelayRequest: (value) =>
                set({ delayRequest: value }),

            savePassword: {},
            setLogin: (login: ILogin | null) => set((state) => ({
                savePassword: login ? {
                    ...state.savePassword,
                    [login.username]: login.password
                } : {
                    ...state.savePassword
                },
                login: login
            })),
            login: null,
            syncProgress: null,
            setSyncProgress: (payload: Record<string, any> | null) => set((state) => ({
                syncProgress: payload ? {
                    completed: 0,
                    total: 0,
                    invoice: null,
                    ...(state.syncProgress || {}),
                    ...payload
                } : null,
            }))
        }),
        {
            name: "app-storage",
            partialize: (state) => ({
                autostartInitialized: state.autostartInitialized,
                delayRequest: state.delayRequest,
                savePassword: state.savePassword
            })
        }
    )
);

export const getTokenTct = () => useAppStore.getState().login?.token;
export const getSavePassword = () => useAppStore.getState().savePassword;
export const getDelayRequest = () => useAppStore.getState().delayRequest;