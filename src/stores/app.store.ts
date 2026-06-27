import { create } from "zustand";
import { persist } from "zustand/middleware";

interface IAppState {
    delayRequest: number;
    setDelayRequest: (delayRequest: number) => void;
    tokenTCT: string | null;
    setTokenTCT: (tokenTCT: string | null) => void;
}

export const useAppStore = create<IAppState>()(
    persist(
        (set) => ({
            delayRequest: 1500,
            setDelayRequest: (value) =>
                set({ delayRequest: value }),
            tokenTCT: null,
            setTokenTCT: (tokenTCT: string | null) => set({ tokenTCT })
        }),
        {
            name: "app-storage",
            partialize: (state) => ({
                delayRequest: state.delayRequest,
            })
        }
    )
);

export const getTokenTct = () => useAppStore.getState().tokenTCT;
export const getDelayRequest = () => useAppStore.getState().delayRequest;