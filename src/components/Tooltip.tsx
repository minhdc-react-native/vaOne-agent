import { ReactNode, useState } from "react";
import { cn } from "../lib/utils";

type TooltipPosition = "top" | "bottom" | "left" | "right";

interface TooltipProps {
    content: ReactNode;
    children: ReactNode;
    position?: TooltipPosition;
    className?: string;
}

export default function Tooltip({
    content,
    children,
    position = "top",
    className,
}: TooltipProps) {
    const [visible, setVisible] = useState(false);

    const positionClass = {
        top: "bottom-full left-1/2 -translate-x-1/2 mb-2",
        bottom: "top-full left-1/2 -translate-x-1/2 mt-2",
        left: "right-full top-1/2 -translate-y-1/2 mr-2",
        right: "left-full top-1/2 -translate-y-1/2 ml-2",
    }[position];

    return (
        <div
            className="relative inline-flex"
            onMouseEnter={() => setVisible(true)}
            onMouseLeave={() => setVisible(false)}
        >
            {children}

            {visible && (
                <div
                    className={cn(
                        "absolute z-50 whitespace-nowrap rounded-md bg-black px-2 py-1 text-xs text-white shadow-lg",
                        "animate-fade-in",
                        positionClass,
                        className
                    )}
                >
                    {content}
                </div>
            )}
        </div>
    );
}