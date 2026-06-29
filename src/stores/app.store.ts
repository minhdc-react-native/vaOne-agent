import { create } from "zustand";
import { persist } from "zustand/middleware";
interface ILoginTct {
    username: string;
    password: string;
    token: string;
}
interface IAppState {
    delayRequest: number;
    setDelayRequest: (delayRequest: number) => void;
    savePasswordLoginTct: Record<string, string>;
    setLoginTct: (loginTct: ILoginTct | null) => void;
    tokenTct: ILoginTct | null;
}

export const useAppStore = create<IAppState>()(
    persist(
        (set) => ({
            delayRequest: 1500,
            setDelayRequest: (value) =>
                set({ delayRequest: value }),
            savePasswordLoginTct: {},
            setLoginTct: (loginTct: ILoginTct | null) => set((state) => ({
                savePasswordLoginTct: loginTct ? {
                    ...state.savePasswordLoginTct,
                    [loginTct.username]: loginTct.password
                } : {
                    ...state.savePasswordLoginTct
                },
                tokenTct: loginTct
            })),
            tokenTct: null
        }),
        {
            name: "app-storage",
            partialize: (state) => ({
                delayRequest: state.delayRequest,
                savePasswordLoginTct: state.savePasswordLoginTct
            })
        }
    )
);

export const getTokenTct = () => useAppStore.getState().tokenTct?.token;
export const getSavePasswordLoginTct = () => useAppStore.getState().savePasswordLoginTct;
export const getDelayRequest = () => useAppStore.getState().delayRequest;