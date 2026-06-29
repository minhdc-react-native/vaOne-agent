import { cn } from "../lib/utils";

interface ProgressProps {
    value: number;
    total: number;
    showLabel?: boolean;
    className?: string;
}

export default function Progress({
    value,
    total,
    showLabel = true,
    className,
}: ProgressProps) {
    const safeTotal = Math.max(total, 1);

    const percent = Math.min(
        100,
        Math.max(0, (value / safeTotal) * 100)
    );

    return (
        <div className={cn("w-full", className)}>
            <div
                className="
                    relative
                    h-5
                    w-full
                    overflow-hidden
                    rounded-full
                    bg-gray-300
                "
            >
                <div
                    className="
                        h-full
                        rounded-full
                        bg-[var(--primary)]
                        transition-all
                        duration-300
                    "
                    style={{
                        width: `${percent}%`,
                    }}
                />

                {showLabel && (
                    <div
                        className="
                            pointer-events-none
                            absolute
                            inset-0
                            flex
                            items-center
                            justify-center
                            text-[11px]
                            font-semibold
                            text-white
                            mix-blend-difference
                        "
                    >
                        {Math.round(percent)}% ({value}/{total})
                    </div>
                )}
            </div>
        </div>
    );
}