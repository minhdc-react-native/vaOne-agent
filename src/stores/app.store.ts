import { create } from "zustand";
import { persist } from "zustand/middleware";
interface ILogin {
    tenantId: string,
    source: string;
    username: string;
    password: string;
    token: string;
    reConnect: boolean;
    taxCode: string;
    idAccount: string;
}
interface IAppState {
    autostartInitialized: boolean,
    setAutostartInitialized: (autostartInitialized: boolean) => void;
    delayRequest: number;
    setDelayRequest: (delayRequest: number) => void;

    savePassword: Record<string, string>;
    setLogin: (login: ILogin | null) => void;
    login: ILogin | null;
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