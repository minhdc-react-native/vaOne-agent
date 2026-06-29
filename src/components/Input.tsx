import {
    forwardRef,
    useState,
    type InputHTMLAttributes,
    type ReactNode,
} from "react";

import { Eye, EyeOff } from "lucide-react";
import { cn } from "../lib/utils";


export interface InputProps
    extends InputHTMLAttributes<HTMLInputElement> {
    label?: string;
    labelPosition?: "top" | "left" | "right";
    error?: string;
    icon?: ReactNode;
    password?: boolean;
}

const Input = forwardRef<HTMLInputElement, InputProps>(
    (
        {
            label,
            labelPosition = "top",
            error,
            icon,
            password = false,
            className,
            type,
            disabled,
            ...props
        },
        ref
    ) => {
        const [showPassword, setShowPassword] = useState(false);

        const inputType = password
            ? showPassword
                ? "text"
                : "password"
            : type;

        const wrapperClass =
            labelPosition === "top"
                ? "flex flex-col gap-1"
                : "flex items-center gap-3";

        const labelClass =
            labelPosition === "top"
                ? "text-sm font-medium"
                : "text-sm font-medium whitespace-nowrap";

        return (
            <div className={cn("w-full", wrapperClass)}>
                {label && (
                    <label className={labelClass}>
                        {label}
                    </label>
                )}

                <div
                    className={cn(
                        "flex h-9 w-full min-w-0 items-center rounded-lg border bg-white px-3",
                        "transition-colors",
                        "focus-within:border-(--primary)",
                        error ? "border-red-500" : "border-(--border)",
                        disabled && "opacity-60"
                    )}
                >
                    {icon && (
                        <span className="mr-2 text-gray-500">
                            {icon}
                        </span>
                    )}

                    <input
                        ref={ref}
                        type={inputType}
                        disabled={disabled}
                        className={cn(
                            "flex-1 min-w-0 bg-transparent text-sm outline-none",
                            "appearance-none",
                            "placeholder:text-gray-400",
                            className
                        )}
                        {...props}
                    />

                    {password && (
                        <button
                            type="button"
                            onClick={() =>
                                setShowPassword((v) => !v)
                            }
                            className="ml-2 flex h-6 w-6 items-center justify-center rounded hover:bg-gray-100"
                        >
                            {showPassword ? (
                                <EyeOff size={16} />
                            ) : (
                                <Eye size={16} />
                            )}
                        </button>
                    )}
                </div>

                {error && (
                    <span className="text-xs text-red-500">
                        {error}
                    </span>
                )}
            </div>
        );
    }
);

Input.displayName = "Input";

export default Input;