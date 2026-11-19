// StatusBadge.tsx
import clsx from "clsx";

export type StatusType =
    | "success"
    | "progress"
    | "pending"
    | "warning"
    | "danger"
    | "continue"
    | "inactive"
    | "draft";

type StatusVariant = "dot" | "text";

interface StatusBadgeProps {
    type?: StatusType;
    variant?: StatusVariant;
    label?: string;
    className?: string;
}

export const STATUS_STYLES: Record<
    StatusType,
    {
        label: string;
        dot: string;
        text: string;
        border: string;
        bg: string;
        bgHover: string;
    }
> = {
    success: {
        label: "Success",
        dot: "bg-green-500",
        text: "text-green-700",
        border: "border-green-200",
        bg: "bg-green-50",
        bgHover: "hover:bg-green-100",
    },

    progress: {
        label: "In Progress",
        dot: "bg-blue-500",
        text: "text-blue-700",
        border: "border-blue-200",
        bg: "bg-blue-50",
        bgHover: "hover:bg-blue-100",
    },

    pending: {
        label: "Pending",
        dot: "bg-purple-500",
        text: "text-purple-700",
        border: "border-purple-200",
        bg: "bg-purple-50",
        bgHover: "hover:bg-purple-100",
    },

    warning: {
        label: "Warning",
        dot: "bg-orange-500",
        text: "text-orange-700",
        border: "border-orange-200",
        bg: "bg-orange-50",
        bgHover: "hover:bg-orange-100",
    },

    danger: {
        label: "Danger",
        dot: "bg-red-500",
        text: "text-red-700",
        border: "border-red-200",
        bg: "bg-red-50",
        bgHover: "hover:bg-red-100",
    },

    continue: {
        label: "Continue",
        dot: "bg-cyan-500",
        text: "text-cyan-700",
        border: "border-cyan-200",
        bg: "bg-cyan-50",
        bgHover: "hover:bg-cyan-100",
    },

    inactive: {
        label: "Inactive",
        dot: "bg-gray-400",
        text: "text-gray-700",
        border: "border-gray-200",
        bg: "bg-gray-50",
        bgHover: "hover:bg-gray-200",
    },

    draft: {
        label: "Draft",
        dot: "bg-slate-400",
        text: "text-slate-700",
        border: "border-slate-200",
        bg: "bg-slate-50",
        bgHover: "hover:bg-slate-200",
    },
};

export default function StatusBadge({
    type = "pending",
    variant = "dot",
    label,
    className,
}: StatusBadgeProps) {
    const style = STATUS_STYLES[type];

    return (
        <span
            className={clsx(
                "inline-flex items-center gap-1 rounded-full border px-2 text-xs font-medium",
                style.bg,
                style.border,
                style.text,
                className
            )}
        >
            {variant === "dot" && (
                <span
                    className={clsx(
                        "h-2 w-2 rounded-full",
                        style.dot
                    )}
                />
            )}
            <span className="flex flex-1 justify-center">{label ?? style.label}</span>
        </span>
    );
}