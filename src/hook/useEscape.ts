import { useEffect } from "react";

export function useEscape(
    handler: (callBack?: {
        onConfirm?: () => Promise<boolean>;
        onClose?: () => void;
    }) => void,
    enabled = true,
    callBack?: {
        onConfirm?: () => Promise<boolean>;
        onClose?: () => void;
    }
) {
    useEffect(() => {
        if (!enabled) return;

        const onKeyDown = (e: KeyboardEvent) => {
            if (e.key !== "Escape") return;

            e.preventDefault();
            e.stopPropagation();

            handler(callBack);
        };

        // true = capture phase, bắt trước handler ở bubble phase
        window.addEventListener("keydown", onKeyDown, true);

        return () => {
            window.removeEventListener("keydown", onKeyDown, true);
        };
    }, [handler, enabled, callBack]);
}
