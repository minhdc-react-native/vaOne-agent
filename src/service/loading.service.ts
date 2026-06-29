import { create } from "zustand";

type LoadingState = {
    visible: boolean;
    text?: string;
};

type LoadingStore = {
    state: LoadingState;
    show: (text?: string) => void;
    hide: () => void;
};

export const useLoading = create<LoadingStore>((set) => ({
    state: {
        visible: false,
        text: "",
    },

    show: (text = "Loading...") =>
        set({
            state: { visible: true, text },
        }),

    hide: () =>
        set({
            state: { visible: false, text: "" },
        }),
}));