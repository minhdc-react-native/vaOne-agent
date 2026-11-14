import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}
export const getCurrentDateTime = (
    type: "date" | "dateTime" | "time" = "date",
): string => {
    const now = new Date();

    const pad = (value: number) => String(value).padStart(2, "0");

    const date = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
    const time = `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())}`;

    switch (type) {
        case "date":
            return date;

        case "dateTime":
            return `${date} ${time}`;

        case "time":
            return time;

        default:
            return date;
    }
};

export const getCurrentMonthDateRange = (): {
    fromDate: string;
    toDate: string;
} => {
    const now = new Date();

    const pad = (value: number) => String(value).padStart(2, "0");

    const formatDate = (date: Date) =>
        `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;

    const firstDay = new Date(now.getFullYear(), now.getMonth(), 1);

    return {
        fromDate: formatDate(firstDay),
        toDate: formatDate(now),
    };
};

export const formatNumber = (
    value: number | string | null | undefined,
    decimals = 0,
): string => {
    if (value === null || value === undefined || value === "") {
        return "";
    }

    const number = typeof value === "number" ? value : Number(value);

    if (!Number.isFinite(number)) {
        return "";
    }

    return new Intl.NumberFormat("vi-VN", {
        minimumFractionDigits: decimals,
        maximumFractionDigits: decimals,
    }).format(number);
};

export const isEmpty = (value: unknown): boolean => {
    if (value === null || value === undefined) {
        return true;
    }

    if (typeof value === "number") {
        return value === 0;
    }

    if (typeof value === "string") {
        return value.trim().length === 0;
    }

    if (Array.isArray(value)) {
        return value.length === 0;
    }

    if (typeof value === "object") {
        return Object.keys(value).length === 0;
    }

    return false;
};

export function delay(ms: number = 3000) {
    return new Promise(resolve => setTimeout(resolve, ms));
}
