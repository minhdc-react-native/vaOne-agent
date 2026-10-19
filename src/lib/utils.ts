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