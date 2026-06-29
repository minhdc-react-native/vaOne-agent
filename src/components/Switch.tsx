import { forwardRef } from "react";
import { cn } from "../lib/utils";

export interface SwitchProps {
    checked: boolean;
    onChange: (checked: boolean) => void;
    label?: string;
    description?: string;
    disabled?: boolean;
    className?: string;
}

const Switch = forwardRef<HTMLButtonElement, SwitchProps>(
    (
        {
            checked,
            onChange,
            label,
            description,
            disabled = false,
            className,
        },
        ref
    ) => {
        return (
            <div
                className={cn(
                    "flex items-center justify-between gap-3",
                    className
                )}
            >
                {(label || description) && (
                    <div className="flex flex-1 flex-col">
                        {label && (
                            <span className="text-sm font-medium">
                                {label}
                            </span>
                        )}

                        {description && (
                            <span className="text-xs text-gray-500">
                                {description}
                            </span>
                        )}
                    </div>
                )}

                <button
                    ref={ref}
                    type="button"
                    role="switch"
                    aria-checked={checked}
                    disabled={disabled}
                    onClick={() => onChange(!checked)}
                    className={cn(
                        "relative inline-flex h-4 w-11 shrink-0 rounded-full",
                        "transition-colors duration-200",
                        "focus:outline-none focus:ring-2 focus:ring-(--primary) focus:ring-offset-2",
                        disabled && "cursor-not-allowed opacity-50",
                        checked
                            ? "bg-(--primary)"
                            : "bg-gray-300"
                    )}
                >
                    <span
                        className={cn(
                            "absolute top-0.5 left-0.5",
                            "h-3 w-3 rounded-full bg-white shadow",
                            "transition-transform duration-200",
                            checked && "translate-x-7"
                        )}
                    />
                </button>
            </div>
        );
    }
);

Switch.displayName = "Switch";

export default Switch;