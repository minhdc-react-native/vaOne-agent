import { forwardRef } from "react";
import { cn } from "../lib/utils";
import clsx from "clsx";

export interface SwitchProps {
    checked: boolean;
    onChange: (checked: boolean) => void;
    label?: string;
    description?: string;
    disabled?: boolean;
    className?: string;
    labelClass?: string;
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
            labelClass
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
                            <span className={clsx("text-sm font-medium", labelClass)}>
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
                        "relative inline-flex h-3 w-7 shrink-0 rounded-full",
                        "transition-colors duration-200",
                        "focus:outline-none focus:ring-2 focus:ring-(--primary) focus:ring-offset-2",
                        disabled ? "cursor-not-allowed opacity-50" : "hover:cursor-pointer",
                        checked
                            ? "bg-(--primary)"
                            : "bg-gray-300"
                    )}
                >
                    <span
                        className={cn(
                            "absolute top-0.5 left-0.5",
                            "h-2 w-2 rounded-full bg-white shadow",
                            "transition-transform duration-200",
                            checked && "translate-x-4"
                        )}
                    />
                </button>
            </div>
        );
    }
);

Switch.displayName = "Switch";

export default Switch;