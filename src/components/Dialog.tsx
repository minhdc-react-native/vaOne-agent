import { ReactNode } from "react";
import { cn } from "../lib/utils";
import {
    CheckCircle,
    XCircle,
    AlertTriangle,
    Info,
    HelpCircle,
} from "lucide-react";
import clsx from "clsx";

type DialogType = "question" | "info" | "success" | "error" | "warning";
export type DialogSize =
    | "xs"
    | "sm"
    | "md"
    | "lg"
    | "xl"
    | "2xl"
    | "3xl"
    | "4xl"
    | "5xl";

const DIALOG_SIZE_CLASSES: Record<DialogSize, string> = {
    xs: "w-full max-w-xs",          // 320px
    sm: "w-full max-w-sm",          // 384px
    md: "w-full max-w-md",          // 448px
    lg: "w-full max-w-lg",          // 512px
    xl: "w-full max-w-xl",          // 576px
    "2xl": "w-full max-w-2xl",      // 672px
    "3xl": "w-full max-w-3xl",      // 768px
    "4xl": "w-full max-w-4xl",      // 896px
    "5xl": "w-full max-w-5xl",      // 1024px
};

interface DialogProps {
    open: boolean;
    type?: DialogType;
    title?: string;
    message?: string;
    icon?: ReactNode;
    content?: ReactNode;
    confirmText?: string;
    cancelText?: string;
    onConfirm?: () => void;
    onCancel?: () => void;
    size?: DialogSize;
}

const typeConfig = {
    question: {
        icon: HelpCircle,
        color: "text-blue-500",
        bg: "bg-blue-50",
    },
    info: {
        icon: Info,
        color: "text-gray-500",
        bg: "bg-gray-50",
    },
    success: {
        icon: CheckCircle,
        color: "text-green-500",
        bg: "bg-green-50",
    },
    error: {
        icon: XCircle,
        color: "text-red-500",
        bg: "bg-red-50",
    },
    warning: {
        icon: AlertTriangle,
        color: "text-yellow-500",
        bg: "bg-yellow-50",
    },
};

export default function Dialog({
    open,
    type = "info",
    size = "md",
    title,
    message,
    confirmText = "OK",
    cancelText = "Cancel",
    onConfirm,
    onCancel,
    content
}: DialogProps) {
    if (!open) return null;

    const config = typeConfig[type];
    const Icon = config.icon;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
            <div className={clsx("rounded-xl bg-white p-5 shadow-lg", DIALOG_SIZE_CLASSES[size])}>
                {/* Header */}
                <div className="flex items-center gap-3">
                    <div className={cn("rounded-full p-2", config.bg)}>
                        <Icon className={cn("h-5 w-5", config.color)} />
                    </div>

                    <div className="text-base font-semibold">
                        {title || type.toUpperCase()}
                    </div>
                </div>

                {/* Message */}
                {message && (
                    <div className="mt-3 text-sm text-gray-600">
                        {message}
                    </div>
                )}
                {content}
                {/* Actions */}
                <div className="mt-5 flex justify-end gap-2">
                    {type === "question" && (
                        <button
                            onClick={onCancel}
                            className="rounded-md px-3 py-1.5 text-sm hover:bg-gray-100"
                        >
                            {cancelText}
                        </button>
                    )}

                    <button
                        onClick={onConfirm}
                        className={cn(
                            "rounded-md px-3 py-1.5 text-sm text-white",
                            type === "error"
                                ? "bg-red-500 hover:bg-red-600"
                                : type === "success"
                                    ? "bg-green-500 hover:bg-green-600"
                                    : type === "warning"
                                        ? "bg-yellow-500 hover:bg-yellow-600"
                                        : "bg-blue-500 hover:bg-blue-600"
                        )}
                    >
                        {confirmText}
                    </button>
                </div>
            </div>
        </div>
    );
}