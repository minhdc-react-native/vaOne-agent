import clsx from "clsx";
import React, { useCallback, useLayoutEffect, useMemo, useRef, useState } from "react";
import { useEffect } from "react";
import ReactDOM from "react-dom";
import { Placement, useAutoPosition } from "../hook/useAutoPosition";
import { popupManager } from "../lib/popupManager";

interface IProps {
    dropdownRef: React.RefObject<HTMLElement>;
    children: React.ReactNode;
    placement?: Placement;
    onClose: () => void;
    isOpen: boolean;
    width?: number;
    className?: string;
    showArrow?: boolean;
    backdrop?: boolean;
    zIndex?: number;
    height?: number;
}
const PopupPosition: React.FC<IProps> = ({
    dropdownRef,
    children,
    placement = "bottom-start",
    onClose,
    isOpen = false,
    width,
    className = '',
    showArrow = false,
    backdrop = false,
    zIndex = 1400,
    height
}) => {
    const [arrowLeft, setArrowLeft] = React.useState<number | null>(null);
    const [arrowTop, setArrowTop] = React.useState<number | null>(null);
    const [realPlacement, setRealPlacement] = React.useState<Placement>(placement);
    const [closing, setClosing] = React.useState(false);
    const [ready, setReady] = React.useState(false);

    const handleClose = () => {
        setClosing(true);
        setReady(false);
        setTimeout(() => {
            onClose?.();
        }, 180);

    };

    const [side] = realPlacement.split("-");

    const popupRef = useRef<any>(null);

    const updateArrow = useCallback(() => {
        const popup = popupRef.current;
        const trigger = dropdownRef.current;

        if (!popup || !trigger) return;

        const popupRect = popup.getBoundingClientRect();
        const triggerRect = trigger.getBoundingClientRect();

        const centerX = triggerRect.left + triggerRect.width / 2;
        const centerY = triggerRect.top + triggerRect.height / 2;

        const arrowX = centerX - popupRect.left;
        const arrowY = centerY - popupRect.top;

        const padding = 12;

        setArrowLeft(
            Math.max(padding, Math.min(arrowX, popupRect.width - padding))
        );
        setArrowTop(
            Math.max(padding, Math.min(arrowY, popupRect.height - padding))
        );

        if (popup.dataset.placement) {
            setRealPlacement(popup.dataset.placement as Placement);
        }
    }, [dropdownRef]);

    const { update } = useAutoPosition(
        {
            referenceRef: dropdownRef,
            popupRef,
            placement,
            afterUpdate: updateArrow,
        },
    );

    const setPopupRef = useCallback((node: HTMLElement | null) => {
        if (node === null) {
            if (popupRef.current) {
                popupRef.current = null;
            }
            return;
        }
        popupRef.current = node;
    }, []);

    const [minWidth, setMinWidth] = useState<number>();

    useLayoutEffect(() => {
        if (dropdownRef.current) {
            setMinWidth(dropdownRef.current.offsetWidth);
        }
    }, [dropdownRef]);

    useLayoutEffect(() => {
        if (!isOpen) return;

        update();
        updateArrow();

    }, [isOpen, update, updateArrow, height]);

    useLayoutEffect(() => {

        if (!isOpen) return;
        // eslint-disable-next-line react-hooks/set-state-in-effect
        setReady(false);
        setClosing(false);

        const id1 =
            requestAnimationFrame(() => {

                update();
                updateArrow();

                const id2 =
                    requestAnimationFrame(() => {

                        setReady(true);

                    });

                return () =>
                    cancelAnimationFrame(id2);
            });

        return () =>
            cancelAnimationFrame(id1);

    }, [isOpen]);

    useEffect(() => {
        if (!isOpen) {
            // eslint-disable-next-line react-hooks/set-state-in-effect
            setReady(false);
        }

        if (!isOpen || !popupRef.current) return;
        const el = popupRef.current;
        popupManager.register(el);
        return () => {
            popupManager.unregister(el);
        };
    }, [isOpen]);

    useEffect(() => {
        const handler = (ev: MouseEvent) => {
            if (!popupRef.current) return;
            const el = popupRef.current;
            if (!popupManager.isTop(el)) return;
            const target = ev.target as Node;
            if (
                el.contains(target) ||
                dropdownRef.current?.contains(target)
            ) {
                return;
            }
            onClose();
        };
        document.addEventListener("mousedown", handler);
        return () => document.removeEventListener("mousedown", handler);
    }, [dropdownRef, onClose]);

    const animationState =
        ready
            ? "enter-active"
            : "enter";

    const sideClass =
        getPopupSideClass(
            side,
            animationState
        );

    const popupClass = clsx(

        "absolute z-1500",

        "transform transition-all duration-180 ease-out",

        "will-change-[transform,opacity]",

        !closing &&
        animationState === "enter" &&
        "opacity-0",

        !closing &&
        animationState === "enter-active" &&
        "opacity-100",

        closing &&
        "opacity-0 scale-95",

        sideClass
    );

    if (!isOpen) return null;
    return ReactDOM.createPortal(
        <>
            {backdrop && (
                <div
                    className={clsx(
                        "fixed inset-0",
                        "bg-black/30",
                        "transition-opacity duration-180",

                        !closing &&
                            animationState === "enter"
                            ? "opacity-0"
                            : "opacity-100",

                        closing &&
                        "opacity-0"
                    )}
                    style={{ zIndex }}
                    onClick={handleClose}
                />
            )}

            <div
                ref={setPopupRef}

                className={popupClass}

                style={{ width, minWidth, zIndex: zIndex + 1 }}
            >
                {showArrow && (

                    <Arrow
                        side={side}
                        arrowLeft={(arrowLeft || 0) - 5}
                        arrowTop={arrowTop || 0}
                    />

                )}

                <div
                    className={clsx(
                        "bg-white",
                        "border border-gray-300",
                        "rounded-sm",
                        "shadow-xl",
                        className
                    )}
                >
                    {children}
                </div>

            </div>
        </>,
        document.body
    );
};

export default PopupPosition;


interface ArrowProps {
    side: string;
    arrowLeft: number;
    arrowTop: number;
}
const Arrow = ({ side, arrowLeft, arrowTop }: ArrowProps) => {
    const size = 8;
    const bg = "#d1d5db"; // gray-300
    const base: React.CSSProperties = {
        position: "absolute",
        width: 0,
        height: 0,
        pointerEvents: "none"
    };

    if (side === "bottom") {
        return (
            <div
                style={{
                    ...base,
                    left: arrowLeft ?? undefined,
                    top: -size,
                    transform: "translateX(-50%)",
                }}
            >
                <div
                    style={{
                        top: 1,
                        left: 0,
                        zIndex: 2,
                        position: "absolute",
                        borderLeft: `${size}px solid transparent`,
                        borderRight: `${size}px solid transparent`,
                        borderBottom: `${size}px solid white`
                    }}
                />
                <div
                    style={{
                        top: 0,
                        left: 0,
                        position: "absolute",
                        zIndex: 1,
                        borderLeft: `${size}px solid transparent`,
                        borderRight: `${size}px solid transparent`,
                        borderBottom: `${size}px solid ${bg}`
                    }}
                />
            </div>
        );
    }

    if (side === "top") {
        return (
            <div
                style={{
                    ...base,
                    left: arrowLeft ?? undefined,
                    bottom: -size,
                    transform: "translateX(-50%)",
                }}
            >
                <div
                    style={{
                        bottom: 1,
                        left: 0,
                        zIndex: 2,
                        position: "absolute",
                        borderLeft: `${size}px solid transparent`,
                        borderRight: `${size}px solid transparent`,
                        borderTop: `${size}px solid white`
                    }}
                />
                <div
                    style={{
                        bottom: 0,
                        left: 0,
                        position: "absolute",
                        zIndex: 1,
                        borderLeft: `${size}px solid transparent`,
                        borderRight: `${size}px solid transparent`,
                        borderTop: `${size}px solid ${bg}`
                    }}
                />
            </div>
        );
    }

    if (side === "left") {
        return (
            <div
                style={{
                    ...base,
                    top: arrowTop ?? undefined,
                    right: -size,
                    transform: "translateY(-50%)",
                }}
            >
                {/* white fill */}
                <div
                    style={{
                        left: 1,
                        top: 0,
                        zIndex: 2,
                        position: "absolute",
                        borderTop: `${size}px solid transparent`,
                        borderBottom: `${size}px solid transparent`,
                        borderLeft: `${size}px solid white`
                    }}
                />

                {/* border */}
                <div
                    style={{
                        left: 0,
                        top: 0,
                        zIndex: 1,
                        position: "absolute",
                        borderTop: `${size}px solid transparent`,
                        borderBottom: `${size}px solid transparent`,
                        borderLeft: `${size}px solid ${bg}`
                    }}
                />
            </div>
        );
    }

    if (side === "right") {
        return (
            <div
                style={{
                    ...base,
                    top: arrowTop ?? undefined,
                    left: -size,
                    transform: "translateY(-50%)",
                }}
            >
                {/* white fill */}
                <div
                    style={{
                        right: 1,
                        top: 0,
                        zIndex: 2,
                        position: "absolute",
                        borderTop: `${size}px solid transparent`,
                        borderBottom: `${size}px solid transparent`,
                        borderRight: `${size}px solid white`
                    }}
                />

                {/* border */}
                <div
                    style={{
                        right: 0,
                        top: 0,
                        zIndex: 1,
                        position: "absolute",
                        borderTop: `${size}px solid transparent`,
                        borderBottom: `${size}px solid transparent`,
                        borderRight: `${size}px solid ${bg}`
                    }}
                />
            </div>
        );
    }

    return null;
};


function getPopupSideClass(
    side?: string,
    state?: string
) {

    const enter =
        state === "enter";

    const active =
        state === "enter-active";

    switch (side) {

        case "top":

            return clsx(
                enter &&
                "-translate-y-2 scale-95",

                active &&
                "translate-y-0 scale-100"
            );

        case "bottom":

            return clsx(
                enter &&
                "translate-y-2 scale-95",

                active &&
                "translate-y-0 scale-100"
            );

        case "left":

            return clsx(
                enter &&
                "-translate-x-2 scale-95",

                active &&
                "translate-x-0 scale-100"
            );

        case "right":

            return clsx(
                enter &&
                "translate-x-2 scale-95",

                active &&
                "translate-x-0 scale-100"
            );

        default:

            return clsx(
                enter && "scale-95",
                active && "scale-100"
            );
    }
}