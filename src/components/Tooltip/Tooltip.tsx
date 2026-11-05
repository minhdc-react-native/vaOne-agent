import { MouseEventHandler, useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import clsx from "clsx";
import { Placement, useAutoPosition } from "../../hook/useAutoPosition";
import { Portal } from "./Portal";
interface ITooltipProps {
    children: React.ReactNode;
    content?: React.ReactNode;
    title?: string;
    subTitle?: string;
    className?: string;
    classTooltip?: string;
    openFirst?: boolean;
    placement?: Placement;
    delay?: number;
    variant?: "dark" | "light";
    onClick?: MouseEventHandler<HTMLDivElement>
}
export function Tooltip({ children, content, title, subTitle, className, classTooltip, onClick, openFirst, placement = "top-center", delay = 0, variant = "dark", }: ITooltipProps) {
    const [open, setOpen] = useState(false);
    const triggerRef = useRef<any>(null);
    const tooltipRef = useRef<any>(null);

    const [arrowLeft, setArrowLeft] = useState<number | null>(null);
    const [arrowTop, setArrowTop] = useState<number | null>(null);
    const [realPlacement, setRealPlacement] = useState<Placement>(placement);

    const [side] = realPlacement.split("-");

    const updateArrow = useCallback(() => {
        const tooltip = tooltipRef.current;
        const trigger = triggerRef.current;

        if (!tooltip || !trigger) return;

        const popupRect = tooltip.getBoundingClientRect();
        const triggerRect = trigger.getBoundingClientRect();

        const centerX = triggerRect.left + triggerRect.width / 2;
        const centerY = triggerRect.top + triggerRect.height / 2;

        const arrowX = centerX - popupRect.left;
        const arrowY = centerY - popupRect.top;

        const padding = 12

        setArrowLeft(
            Math.max(padding, Math.min(arrowX, popupRect.width - padding))
        );

        setArrowTop(
            Math.max(padding, Math.min(arrowY, popupRect.height - padding))
        );

        if (tooltip.dataset.placement) {
            setRealPlacement(tooltip.dataset.placement as Placement);
        }
    }, [triggerRef]);

    const { update } = useAutoPosition({
        referenceRef: triggerRef,
        popupRef: tooltipRef,
        placement,
        afterUpdate: updateArrow,
    });

    useLayoutEffect(() => {
        if (!open) return;
        update();
        updateArrow();
    }, [open, update, updateArrow]);

    useEffect(() => {
        if (!title || !openFirst) return;
        // eslint-disable-next-line react-hooks/set-state-in-effect
        setOpen(true);
        const t = setTimeout(() => setOpen(false), 3000);
        return () => clearTimeout(t);
    }, [title, openFirst]);

    const hoverTimerRef = useRef<number | null>(null);

    const handleMouseEnter = (e: React.MouseEvent) => {
        const target = e.target as HTMLElement;

        // Nếu target thuộc một Tooltip con thì Tooltip cha không mở
        const childTooltip = target.closest("[data-tooltip]");

        if (childTooltip && childTooltip !== e.currentTarget) {
            return;
        }

        if (hoverTimerRef.current) {
            clearTimeout(hoverTimerRef.current);
            hoverTimerRef.current = null;
        }

        hoverTimerRef.current = window.setTimeout(() => {
            setOpen(true);
        }, delay);
    };

    const handleMouseLeave = (e: React.MouseEvent) => {
        const relatedTarget = e.relatedTarget;

        if (
            relatedTarget instanceof Node &&
            triggerRef.current?.contains(relatedTarget)
        ) {
            return;
        }

        if (hoverTimerRef.current) {
            clearTimeout(hoverTimerRef.current);
            hoverTimerRef.current = null;
        }

        setOpen(false);
    };

    const handleMouseOver = (e: React.MouseEvent) => {
        const target = e.target as HTMLElement;

        const tooltip = target.closest("[data-tooltip]");

        // Đang ở Tooltip con
        if (tooltip && tooltip !== triggerRef.current) {
            if (hoverTimerRef.current) {
                clearTimeout(hoverTimerRef.current);
                hoverTimerRef.current = null;
            }

            setOpen(false);
            return;
        }

        // Đang quay lại vùng của Tooltip cha
        if (tooltip === triggerRef.current) {
            if (hoverTimerRef.current) {
                clearTimeout(hoverTimerRef.current);
            }

            hoverTimerRef.current = window.setTimeout(() => {
                setOpen(true);
            }, delay);
        }
    };

    useEffect(() => {
        return () => {
            if (hoverTimerRef.current) {
                clearTimeout(hoverTimerRef.current);
            }
        };
    }, []);


    return (
        <div
            ref={triggerRef}
            data-tooltip
            className={`relative ${className || ''}`}
            onMouseEnter={handleMouseEnter}
            onMouseOver={handleMouseOver}
            onMouseLeave={handleMouseLeave}
            onClick={onClick}
        >
            {children}

            {open && (
                <Portal zIndex={2000}>
                    <div
                        ref={tooltipRef}
                        className={clsx(
                            "fixed",
                            "pointer-events-none",
                            "max-w-125",
                            "wrap-break-word",
                            "rounded-sm px-2 py-1 text-xs",
                            variant === "dark"
                                ? "bg-gray-900 text-white border-[0.5px] border-white"
                                : "border-[0.5px] border-gray-300 bg-white text-gray-700 shadow-md",
                            classTooltip
                        )}
                    >
                        {(side === "top" || side === "bottom") && arrowLeft !== null && (
                            <div
                                className={clsx(
                                    "absolute h-0 w-0",
                                    side === "top" && [
                                        "border-l-4 border-r-4 border-t-4 border-transparent",
                                        variant === "dark"
                                            ? "border-t-gray-900"
                                            : "border-t-white",
                                    ],
                                    side === "bottom" && [
                                        "border-l-4 border-r-4 border-b-4 border-transparent",
                                        variant === "dark"
                                            ? "border-b-gray-900"
                                            : "border-b-white",
                                    ]
                                )}
                                style={{
                                    left: arrowLeft,
                                    transform: "translateX(-50%)",
                                    bottom: side === "top" ? "-4px" : undefined,
                                    top: side === "bottom" ? "-4px" : undefined,
                                }}
                            />
                        )}

                        {(side === "left" || side === "right") && arrowTop !== null && (
                            <div
                                className={clsx(
                                    "absolute h-0 w-0",
                                    side === "left" && [
                                        "border-t-4 border-b-4 border-l-4 border-transparent",
                                        variant === "dark"
                                            ? "border-l-gray-900"
                                            : "border-l-white",
                                    ],
                                    side === "right" && [
                                        "border-t-4 border-b-4 border-r-4 border-transparent",
                                        variant === "dark"
                                            ? "border-r-gray-900"
                                            : "border-r-white",
                                    ]
                                )}
                                style={{
                                    top: arrowTop,
                                    transform: "translateY(-50%)",
                                    right: side === "left" ? "-4px" : undefined,
                                    left: side === "right" ? "-4px" : undefined,
                                }}
                            />
                        )}

                        <div className="flex flex-col items-center w-full max-w-[320px] mx-auto text-center">
                            {title && (
                                <div className="w-full max-w-full wrap-break-word whitespace-normal">
                                    {title}
                                </div>
                            )}

                            {subTitle && (
                                <div className={`w-full max-w-full wrap-break-word whitespace-normal ${variant === "dark" ? "text-yellow-100" : "text-yellow-700"
                                    }`}>
                                    {subTitle}
                                </div>
                            )}

                            {content}
                        </div>
                    </div>
                </Portal>
            )}
        </div>
    );
}
