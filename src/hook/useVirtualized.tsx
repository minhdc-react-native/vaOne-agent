import { useCallback, useEffect, useMemo, useState } from "react";

export interface IUseVirtualizedOptions<T> {
    data: T[];
    rowHeight: number;
    overscan?: number;
    containerRef: React.RefObject<HTMLDivElement | null>;
}

export function useVirtualized<T>({
    data,
    rowHeight,
    overscan = 10,
    containerRef
}: IUseVirtualizedOptions<T>) {
    const [scrollTop, setScrollTop] = useState(0);
    const [containerHeight, setContainerHeight] = useState(0);

    // theo dõi scroll
    useEffect(() => {
        const el = containerRef.current;
        if (!el) return;

        const onScroll = () => setScrollTop(el.scrollTop);

        setContainerHeight(el.clientHeight);
        el.addEventListener("scroll", onScroll);

        return () => el.removeEventListener("scroll", onScroll);
    }, [containerRef]);

    const total = data.length;

    const startIndex = Math.max(
        0,
        Math.floor(scrollTop / rowHeight) - overscan
    );

    const visibleCount = Math.ceil(containerHeight / rowHeight) + overscan * 2;

    const endIndex = Math.min(
        total,
        startIndex + visibleCount
    );

    const visibleData = useMemo(
        () => data.slice(startIndex, endIndex),
        [data, startIndex, endIndex]
    );

    const offsetTop = startIndex * rowHeight;
    const offsetBottom = (total - endIndex) * rowHeight;

    const scrollToBottom = useCallback(() => {
        const el = containerRef.current;
        if (!el) return;

        el.scrollTop = el.scrollHeight;
    }, [containerRef]);

    return useMemo(
        () => ({
            data,
            visibleData,
            offsetTop,
            offsetBottom,
            startIndex,
            endIndex,
            scrollToBottom
        }),
        [
            data,
            visibleData,
            offsetTop,
            offsetBottom,
            startIndex,
            endIndex,
            scrollToBottom
        ]
    );
}
