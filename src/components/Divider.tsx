import { cn } from "../lib/utils";

interface DividerProps {
    orientation?: "horizontal" | "vertical";
    text?: string;
    className?: string;
}

export function Divider({
    orientation = "horizontal",
    text,
    className,
}: DividerProps) {
    if (orientation === "vertical") {
        return (
            <div
                className={cn(
                    "w-px self-stretch bg-border bg-gray-200",
                    className
                )}
            />
        );
    }

    if (!text) {
        return (
            <div
                className={cn(
                    "h-px w-full bg-border bg-gray-200",
                    className
                )}
            />
        );
    }

    return (
        <div
            className={cn(
                "flex items-center gap-3",
                className
            )}
        >
            <div className="h-px flex-1 bg-border bg-gray-200" />
            <span className="text-xs text-muted-foreground whitespace-nowrap">
                {text}
            </span>
            <div className="h-px flex-1 bg-border bg-gray-200" />
        </div>
    );
}