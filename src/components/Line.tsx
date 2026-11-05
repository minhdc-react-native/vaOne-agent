import React from "react";
import clsx from "clsx";

export type LineOrientation = "horizontal" | "vertical";
export type LineAlign = "left" | "center" | "right";

interface LineProps extends React.HTMLAttributes<HTMLDivElement> {
    orientation?: LineOrientation;
    thickness?: number;
    length?: string | number;
    color?: string;

    text?: React.ReactNode;
    align?: LineAlign; // vị trí text
}

export const Line: React.FC<LineProps> = ({
    orientation = "horizontal",
    thickness = 1,
    length = "100%",
    color = "#e5e7eb",
    text,
    align = "center",
    style,
    className,
    ...rest
}) => {
    const isHorizontal = orientation === "horizontal";

    const sizeStyle: React.CSSProperties = isHorizontal
        ? {
            width: typeof length === "number" ? `${length}px` : length,
        }
        : {
            height: typeof length === "number" ? `${length}px` : length,
        };

    const lineCommon: React.CSSProperties = {
        backgroundColor: color,
    };

    const lineStyle = isHorizontal
        ? { height: thickness }
        : { width: thickness };

    // không có text → render như cũ
    if (!text) {
        return (
            <div
                role="separator"
                aria-orientation={orientation}
                className={clsx("ui-line", className)}
                style={{
                    ...lineCommon,
                    ...lineStyle,
                    ...sizeStyle,
                    ...style,
                }}
                {...rest}
            />
        );
    }

    return (
        <div
            role="separator"
            aria-orientation={orientation}
            className={clsx(
                "ui-line flex items-center",
                isHorizontal ? "flex-row" : "flex-col",
                className
            )}
            style={sizeStyle}
            {...rest}
        >
            {/* BEFORE LINE */}
            <div
                style={{
                    ...lineCommon,
                    ...lineStyle,
                    ...(isHorizontal
                        ? {
                            flex:
                                align === "center"
                                    ? 1
                                    : align === "left"
                                        ? "0 0 40px"
                                        : 1,
                        }
                        : {
                            flex:
                                align === "center"
                                    ? 1
                                    : align === "left"
                                        ? "0 0 40px"
                                        : 1,
                        }),
                }}
            />

            {/* TEXT */}
            <span
                className="px-2 whitespace-nowrap text-gray-500"
                style={
                    isHorizontal
                        ? {}
                        : {
                            writingMode: "vertical-rl",
                            transform: "rotate(180deg)",
                        }
                }
            >
                {text}
            </span>

            {/* AFTER LINE */}
            <div
                style={{
                    ...lineCommon,
                    ...lineStyle,
                    ...(isHorizontal
                        ? {
                            flex:
                                align === "center"
                                    ? 1
                                    : align === "right"
                                        ? "0 0 40px"
                                        : 1,
                        }
                        : {
                            flex:
                                align === "center"
                                    ? 1
                                    : align === "right"
                                        ? "0 0 40px"
                                        : 1,
                        }),
                }}
            />
        </div>
    );
};