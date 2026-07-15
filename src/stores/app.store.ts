import { create } from "zustand";
import { persist } from "zustand/middleware";
interface ILogin {
    username: string;
    password: string;
    token: string;
    taxCode?: string;
    idAccount?: string;
}
interface IAppState {
    autostartInitialized: boolean,
    setAutostartInitialized: (autostartInitialized: boolean) => void;
    delayRequest: number;
    setDelayRequest: (delayRequest: number) => void;

    savePasswordLoginTct: Record<string, string>;
    setLoginTct: (login: ILogin | null) => void;
    tokenTct: ILogin | null;

    savePasswordLoginMInvoice: Record<string, string>;
    setLoginMInvoice: (login: ILogin | null) => void;
    tokenMInvoice: ILogin | null;

    savePasswordLoginSaveInvoice: Record<string, string>;
    setLoginSaveInvoice: (login: ILogin | null) => void;
    tokenSaveInvoice: ILogin | null;
}

export const useAppStore = create<IAppState>()(
    persist(
        (set) => ({
            autostartInitialized: false,
            setAutostartInitialized: (autostartInitialized) => set({ autostartInitialized }),
            delayRequest: 1500,
            setDelayRequest: (value) =>
                set({ delayRequest: value }),

            savePasswordLoginTct: {},
            setLoginTct: (login: ILogin | null) => set((state) => ({
                savePasswordLoginTct: login ? {
                    ...state.savePasswordLoginTct,
                    [login.username]: login.password
                } : {
                    ...state.savePasswordLoginTct
                },
                tokenTct: login
            })),
            tokenTct: null,

            savePasswordLoginMInvoice: {},
            setLoginMInvoice: (login: ILogin | null) => set((state) => ({
                savePasswordLoginMInvoice: login ? {
                    ...state.savePasswordLoginMInvoice,
                    [login.username]: login.password
                } : {
                    ...state.savePasswordLoginMInvoice
                },
                tokenMInvoice: login
            })),
            tokenMInvoice: null,

            savePasswordLoginSaveInvoice: {},
            setLoginSaveInvoice: (login: ILogin | null) => set((state) => ({
                savePasswordLoginSaveInvoice: login ? {
                    ...state.savePasswordLoginSaveInvoice,
                    [login.username]: login.password
                } : {
                    ...state.savePasswordLoginSaveInvoice
                },
                tokenSaveInvoice: login
            })),
            tokenSaveInvoice: null
        }),
        {
            name: "app-storage",
            partialize: (state) => ({
                autostartInitialized: state.autostartInitialized,
                delayRequest: state.delayRequest,
                savePasswordLoginTct: state.savePasswordLoginTct
            })
        }
    )
);

export const getTokenTct = () => useAppStore.getState().tokenTct?.token;
export const getSavePasswordLoginTct = () => useAppStore.getState().savePasswordLoginTct;
export const getDelayRequest = () => useAppStore.getState().delayRequest;