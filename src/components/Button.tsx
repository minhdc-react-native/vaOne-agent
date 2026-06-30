import {
    forwardRef,
    type ButtonHTMLAttributes,
    type ReactNode,
} from "react";
import { LoaderCircle } from "lucide-react";
import { cn } from "../lib/utils";

export interface ButtonProps
    extends ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: "primary" | "secondary" | "danger" | "ghost";

    loading?: boolean;

    icon?: ReactNode;

    iconOnly?: boolean;

    iconPosition?: "left" | "right";
}

const Button = forwardRef<HTMLButtonElement, ButtonProps>(
    (
        {
            children,
            className,

            variant = "primary",

            loading = false,

            icon,

            iconOnly = false,

            iconPosition = "left",

            disabled,

            ...props
        },
        ref
    ) => {
        return (
            <button
                ref={ref}
                disabled={disabled || loading}
                className={cn(
                    "inline-flex items-center justify-center",
                    "gap-2",
                    iconOnly ? "rounded-full" : "rounded-lg",
                    !iconOnly && "h-8",
                    "text-sm",
                    "font-medium",
                    "transition-all",
                    "duration-150",
                    "select-none",
                    "active:scale-[0.98]",
                    "disabled:pointer-events-none",
                    "disabled:opacity-50",
                    "hover:cursor-pointer",
                    iconOnly ? "p-1 " : "px-4",

                    variant === "primary" &&
                    "bg-(--primary) text-white hover:bg-(--primary-hover)",

                    variant === "secondary" &&
                    "bg-(--secondary) hover:bg-(--secondary-hover)",

                    variant === "danger" &&
                    "bg-(--danger) text-white hover:bg-(--danger-hover)",

                    variant === "ghost" &&
                    "hover:bg-gray-100",
                    "focus:ring-0 focus:outline-none focus-visible:ring-1 focus-visible:ring-gray-300",
                    className
                )}
                {...props}
            >
                {loading ? (
                    <LoaderCircle
                        size={16}
                        className="animate-spin"
                    />
                ) : (
                    <>
                        {icon && iconPosition === "left" && icon}

                        {!iconOnly && children}

                        {icon && iconPosition === "right" && icon}
                    </>
                )}
            </button>
        );
    }
);

Button.displayName = "Button";

export default Button;