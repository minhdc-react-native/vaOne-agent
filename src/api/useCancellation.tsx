import { useCallback, useEffect, useRef } from "react";

export function useCancellation() {
    const cancelledRef = useRef(false);

    useEffect(() => {
        cancelledRef.current = false;

        return () => {
            cancelledRef.current = true;
        };
    }, []);

    const isCancelled = useCallback(() => cancelledRef.current, []);

    const throwIfCancelled = useCallback(() => {
        if (cancelledRef.current) {
            throw new Error("__CANCELLED__");
        }
    }, []);

    return {
        isCancelled,
        throwIfCancelled,
    };
}