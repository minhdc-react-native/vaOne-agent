import { forwardRef } from "react";
import { cn } from "../lib/utils";

export interface CheckboxProps {
    checked: boolean;
    onChange: (checked: boolean) => void;
    label?: string;
    description?: string;
    disabled?: boolean;
    className?: string;
}

const Checkbox = forwardRef<HTMLInputElement, CheckboxProps>(
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
            <label
                className={cn(
                    "flex items-center justify-between gap-1",
                    disabled ? "cursor-not-allowed opacity-50" : "cursor-pointer",
                    className
                )}
            >
                {(label || description) && (
                    <div className="flex min-w-0 flex-1 flex-col">
                        {label && (
                            <span className="text-xs">
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

                <input
                    ref={ref}
                    type="checkbox"
                    checked={checked}
                    disabled={disabled}
                    onChange={(e) => onChange(e.target.checked)}
                    className={cn(
                        "size-4 shrink-0 cursor-pointer appearance-none",
                        "rounded border border-gray-300 bg-white",
                        "checked:border-(--primary) checked:bg-(--primary)",
                        "focus:outline-none focus:ring-2",
                        "focus:ring-(--primary) focus:ring-offset-2",
                        "disabled:cursor-not-allowed",
                        "relative",
                        "after:absolute after:inset-0",
                        "after:flex after:items-center after:justify-center",
                        "after:text-xs after:font-bold after:text-white",
                        "checked:after:content-['✓']"
                    )}
                />
            </label>
        );
    }
);

Checkbox.displayName = "Checkbox";

export default Checkbox;