import { useCallback, useLayoutEffect } from "react";
export type Placement =
    | "bottom-start"
    | "bottom-center"
    | "bottom-end"
    | "top-start"
    | "top-center"
    | "top-end"
    | "left-start"
    | "left-center"
    | "left-end"
    | "right-start"
    | "right-center"
    | "right-end";

interface AutoPositionProps {
    referenceRef: React.RefObject<HTMLElement> | null;
    popupRef: React.RefObject<HTMLElement> | null;
    placement: Placement;
    afterUpdate?: () => void
}

export function useAutoPosition({ referenceRef, popupRef, placement, afterUpdate }: AutoPositionProps) {
    const update = useCallback(() => {
        autoPositionPopup({
            referenceEl: referenceRef?.current || null,
            popupEl: popupRef?.current || null,
            placement,
        });
        afterUpdate?.();
    }, [placement, referenceRef, popupRef, afterUpdate]);

    useLayoutEffect(() => {
        const onScrollTest = () => update();
        const onResize = () => update();
        window.addEventListener("scroll", onScrollTest, true);
        window.addEventListener("resize", onResize);

        return () => {
            window.removeEventListener("scroll", onScrollTest);
            window.removeEventListener("resize", onResize);
        };
    }, [update]);

    return { update };
}

interface AutoPositionParams {
    referenceEl: HTMLElement | null;
    popupEl: HTMLElement | null;
    placement?: Placement;
    offset?: number;
    padding?: number;
}
function autoPositionPopup({
    referenceEl,
    popupEl,
    placement = "bottom-start",
    offset = 1,
    padding = 4,
}: AutoPositionParams) {
    if (!referenceEl || !popupEl) return;

    const refRect = referenceEl.getBoundingClientRect();
    const popupWidth = popupEl.offsetWidth;
    const popupHeight = popupEl.offsetHeight;

    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    let top = 0;
    let left = 0;
    let finalPlacement = placement;
    const basePositions = {
        "bottom-start": { top: refRect.bottom + offset, left: refRect.left },
        "bottom-center": {
            top: refRect.bottom + offset,
            left: refRect.left + refRect.width / 2 - popupWidth / 2,
        },
        "bottom-end": {
            top: refRect.bottom + offset,
            left: refRect.right - popupWidth,
        },

        "top-start": {
            top: refRect.top - popupHeight - offset,
            left: refRect.left,
        },
        "top-center": {
            top: refRect.top - popupHeight - offset,
            left: refRect.left + refRect.width / 2 - popupWidth / 2,
        },
        "top-end": {
            top: refRect.top - popupHeight - offset,
            left: refRect.right - popupWidth,
        },

        "left-start": {
            top: refRect.top,
            left: refRect.left - popupWidth - offset,
        },
        "left-center": {
            top: refRect.top + refRect.height / 2 - popupHeight / 2,
            left: refRect.left - popupWidth - offset,
        },
        "left-end": {
            top: refRect.bottom - popupHeight,
            left: refRect.left - popupWidth - offset,
        },

        "right-start": {
            top: refRect.top,
            left: refRect.right + offset,
        },
        "right-center": {
            top: refRect.top + refRect.height / 2 - popupHeight / 2,
            left: refRect.right + offset,
        },
        "right-end": {
            top: refRect.bottom - popupHeight,
            left: refRect.right + offset,
        },
    } as const;

    ({ top, left } = basePositions[placement]);

    const spaceBelow = viewportHeight - refRect.bottom;
    const spaceAbove = refRect.top;
    const spaceLeft = refRect.left;
    const spaceRight = viewportWidth - refRect.right;

    if (placement.startsWith("left") && spaceLeft < popupWidth) {
        left = refRect.right + offset;
        finalPlacement = placement.replace("left", "right") as Placement;
    }

    if (placement.startsWith("right") && spaceRight < popupWidth) {
        left = refRect.left - popupWidth - offset;
        finalPlacement = placement.replace("right", "left") as Placement;
    }

    if (placement.startsWith("bottom") && spaceBelow < popupHeight) {
        top = refRect.top - popupHeight - offset;
        finalPlacement = placement.replace("bottom", "top") as Placement;
    }

    if (placement.startsWith("top") && spaceAbove < popupHeight) {
        top = refRect.bottom + offset;
        finalPlacement = placement.replace("top", "bottom") as Placement;
    }

    if (left < padding) left = padding;
    if (left + popupWidth > viewportWidth - padding) {
        left = viewportWidth - popupWidth - padding;
    }

    if (top < padding) top = padding;
    if (top + popupHeight > viewportHeight - padding) {
        top = viewportHeight - popupHeight - padding;
    }

    Object.assign(popupEl.style, {
        position: "fixed",
        top: `${Math.round(top)}px`,
        left: `${Math.round(left)}px`,
    });

    popupEl.dataset.placement = finalPlacement;
}
