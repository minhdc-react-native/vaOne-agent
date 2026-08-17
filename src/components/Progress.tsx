import clsx from "clsx";
import { Loader2 } from "lucide-react";
import { useId, useLayoutEffect, useRef, useState } from "react";

interface ProgressProps {
    value: number;
    total?: number;
    showLabel?: boolean;
    className?: string;
    message?: string;
    unit?: string;
}

export default function Progress({
    value,
    total,
    showLabel = true,
    className,
    message = "...",
    unit = ""
}: ProgressProps) {
    const progressRef = useRef<HTMLDivElement>(null);

    const [size, setSize] = useState({
        width: 0,
        height: 0,
    });

    const clipId = useId();

    const fillRef = useRef<HTMLDivElement>(null);
    const [progressWidth, setProgressWidth] = useState(0);

    useLayoutEffect(() => {
        const updateContainer = () => {
            if (!progressRef.current) return;

            setSize({
                width: progressRef.current.clientWidth,
                height: progressRef.current.clientHeight,
            });
        };

        const updateProgress = () => {
            if (!fillRef.current) return;

            setProgressWidth(fillRef.current.clientWidth);
        };

        // Update lần đầu
        updateContainer();
        updateProgress();

        const containerObserver = new ResizeObserver(updateContainer);
        const progressObserver = new ResizeObserver(updateProgress);

        if (progressRef.current) {
            containerObserver.observe(progressRef.current);
        }

        if (fillRef.current) {
            progressObserver.observe(fillRef.current);
        }

        return () => {
            containerObserver.disconnect();
            progressObserver.disconnect();
        };
    }, []);

    const safeTotal = Math.max(total ?? 1, 1);

    const percent = Math.min(
        100,
        Math.max(0, (value / safeTotal) * 100)
    );

    const label =
        (total != null && total != 0)
            ? `${Math.round(percent)}% (${value}/${total}${unit})`
            : message

    // const progressWidth = size.width * percent / 100;

    return (
        <div className={clsx("w-full", className)}>
            <div
                ref={progressRef}
                className="relative h-5 w-full overflow-hidden rounded-full bg-gray-300"
            >
                {/* Progress */}
                <div
                    ref={fillRef}
                    className="h-full rounded-full bg-red-600 transition-all duration-300"
                    style={{
                        width: `${percent}%`,
                    }}
                />

                {showLabel && (
                    <>
                        {(total != null && total != 0) ? (
                            size.width > 0 && (
                                <svg
                                    className="pointer-events-none absolute inset-0"
                                    width={size.width}
                                    height={size.height}
                                    viewBox={`0 0 ${size.width} ${size.height}`}
                                >
                                    <defs>
                                        <clipPath id={clipId} clipPathUnits="userSpaceOnUse">
                                            <rect
                                                x={0}
                                                y={0}
                                                width={progressWidth}
                                                height={size.height}
                                            />
                                        </clipPath>
                                    </defs>

                                    {/* Text màu đen */}
                                    <text
                                        x={size.width / 2}
                                        y={size.height / 2}
                                        textAnchor="middle"
                                        dominantBaseline="central"
                                        fontSize={11}
                                        fontWeight={600}
                                        fill="#1f2937"
                                    >
                                        {label}
                                    </text>

                                    {/* Text màu trắng */}
                                    <g clipPath={`url(#${clipId})`}>
                                        <text
                                            x={size.width / 2}
                                            y={size.height / 2}
                                            textAnchor="middle"
                                            dominantBaseline="central"
                                            fontSize={11}
                                            fontWeight={600}
                                            fill="white"
                                        >
                                            {label}
                                        </text>
                                    </g>
                                </svg>
                            )
                        ) : (
                            <div className="pointer-events-none absolute inset-0 flex items-center justify-center gap-1 text-black">
                                <Loader2 className="h-4 w-4 animate-spin" />
                                <span className="text-[11px] font-semibold">
                                    {message}
                                </span>
                            </div>
                        )}
                    </>
                )}
            </div>
        </div>
    );
}