import { forwardRef, useCallback, useEffect, useMemo, useRef, useState, type KeyboardEvent, type ReactNode } from "react";
import { IconLucide } from "../IconLucide";
import Input from "../Input";
import PopupPosition from "../PopupPosition";
import Calendar from "./Calendar";
import clsx from "clsx";

interface DateInputProps {
    name?: string;
    value?: string | null;
    onChange?: (value?: string | null) => void;
    label?: string;
    tooltipLabel?: { title?: string, subTitle?: string, content?: ReactNode };
    labelPosition?: "top" | "left" | "right";
    labelClassName?: string;
    labelWidth?: number;
    width?: number;
    disabled?: boolean;
    required?: boolean;
    autoFocus?: boolean;
    isCellEdit?: boolean;
    mode?: "date" | "datetime";
    minuteStep?: 1 | 5 | 10;
    sizeInput?: 'sm' | 'xs';
    isError?: boolean;
    className?: string;
}
export const DateInput = forwardRef<
    HTMLInputElement | HTMLTextAreaElement,
    DateInputProps
>((props, ref) => {
    const inputRef = useRef<HTMLInputElement | HTMLTextAreaElement>(null);
    const {
        value,
        name,
        onChange,
        label,
        labelPosition,
        disabled,
        required,
        autoFocus,
        mode = "date",
        width,
        sizeInput = "sm",
        className
    } = props;

    const mergedRef = useCallback(
        (element: HTMLInputElement | HTMLTextAreaElement | null) => {
            inputRef.current = element;
            if (typeof ref === "function") {
                ref(element);
            } else if (ref) {
                ref.current = element;
            }
        },
        [ref]
    );

    const wrapperRef = useRef<HTMLDivElement>(null);
    const [open, setOpen] = useState(false);
    const [openTime, setOpenTime] = useState(false);
    const [inputValue, setInputValue] = useState(getInputValue(value, mode));

    const timeValue = useMemo(() => {
        if (
            mode !== "datetime" || !value
        ) {
            return "";
        }
        const d = new Date(value);
        return `${String(
            d.getHours()
        ).padStart(2, "0")}:${String(
            d.getMinutes()
        ).padStart(2, "0")}`;
    }, [value, mode]);

    const currentValue = useRef(getInputValue(value, mode))

    useEffect(() => {
        const newValue = getInputValue(value, mode);
        if (currentValue.current === newValue) return;
        currentValue.current = newValue;
        setInputValue(newValue);
    }, [value, mode]);

    const onChangeFull = useCallback((nextInputValue: string) => {
        if (nextInputValue.length < 10) return;
        const parsed = parseDisplayDate(nextInputValue);
        const nextValue = serializeDate(
            parsed!,
            mode,
            timeValue
        );
        if (nextValue !== value) {
            onChange?.(nextValue);
        }
    }, [mode, timeValue, value, onChange]);


    // gõ tay
    const handleInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
        let v = e.target.value.replace(/[^\d/]/g, "");
        // auto slash cho dd/MM
        if (v.length === 2 || v.length === 5) {
            if (!v.endsWith("/")) v += "/";
        }
        if (v.length > 10) v = v.slice(0, 10);
        setInputValue(v);
        onChangeFull(v);
    }, [onChangeFull]);


    const commitValue = useCallback(() => {
        const parsed = parseDisplayDate(inputValue);
        if (!parsed) {
            if (value !== null) {
                onChange?.(null);
            }
            setOpen(false);
            setInputValue("");
            return;
        }
        const nextValue = serializeDate(
            parsed,
            mode,
            timeValue
        );

        if (nextValue !== value) {
            onChange?.(nextValue);
        }
        setInputValue(formatDateDisplay(parsed));
        setOpen(false);
    }, [inputValue, mode, onChange, timeValue, value]);

    // chọn từ popup
    const handleSelectDate = useCallback((date: Date) => {
        setInputValue(formatDateDisplay(date));
        onChange?.(serializeDate(date, mode, timeValue));
        setOpen(false);
    }, [mode, onChange, timeValue]);

    const onKeyDown = useCallback((e: KeyboardEvent<HTMLInputElement>) => {
        if (e.key === "Tab" || e.key === "Enter") {
            commitValue();
        }
    }, [commitValue]);

    const onSelect = useCallback((d: Date) => {
        handleSelectDate(d);
        setOpen(false);
    }, [handleSelectDate]);

    const onOpenChange = useCallback((type: "date" | "time") => {
        if (type === "date") {
            if (openTime) setOpenTime(false);
            setOpen(!open);
        } else {
            if (open) setOpen(false);
            setOpenTime(!openTime);
        }
    }, [open, openTime]);

    return (
        <div
            ref={wrapperRef}
            className={clsx("relative flex w-full min-w-0 flex-row", className)}
        >
            <div className="relative w-full min-w-0 flex-1">
                <Input
                    ref={mergedRef}
                    label={label}
                    name={name}
                    width={width}
                    sizeInput={sizeInput}
                    autoFocus={autoFocus}
                    required={required}
                    disabled={disabled}
                    value={inputValue}
                    placeholder="dd/mm/yyyy"
                    onChange={handleInputChange}
                    onKeyDown={onKeyDown}
                    labelPosition={labelPosition}
                />
                <div className="absolute bottom-1.5 right-0">
                    <IconLucide name="CalendarDays" size={16}
                        className="mx-1 hover:cursor-pointer text-primary opacity-60"
                        onClick={() => !disabled && onOpenChange("date")} />
                </div>
            </div>

            <PopupPosition
                dropdownRef={wrapperRef as any}
                isOpen={open} placement="bottom-end"
                onClose={() => setOpen(false)}
                className="p-2 bg-white shadow rounded"
            >
                <Calendar
                    value={value ? new Date(value) : null}
                    onSelect={onSelect}
                />
            </PopupPosition>
        </div>
    );
});

function getInputValue(value: string | null | undefined, mode: "date" | "datetime") {
    if (!value) {
        return "";
    }
    const date = parseValue(value, mode);
    if (!date) {
        return "";
    }
    return formatDateDisplay(date);
}

function formatDateDisplay(date: Date | null) {
    if (!date) return "";
    const d = String(date.getDate()).padStart(2, "0");
    const m = String(date.getMonth() + 1).padStart(2, "0");
    const y = date.getFullYear();
    return `${d}/${m}/${y}`;
}

function parseDisplayDate(value: string): Date | null {
    const parts = value.split("/");
    if (parts.length !== 3) return null;

    const d = Number(parts[0]);
    const m = Number(parts[1]);
    const yRaw = parts[2] || '';

    if (!d || !m) return null;

    const y = normalizeYear(yRaw);

    const date = new Date(y, m - 1, d);

    if (
        date.getDate() !== d ||
        date.getMonth() !== m - 1 ||
        date.getFullYear() !== y
    ) {
        return null;
    }

    return date;
}


function normalizeYear(inputYear: string): number {
    const currentYear = new Date().getFullYear(); // ví dụ 2026

    if (inputYear.length >= 4) return Number(inputYear);

    const currentStr = String(currentYear);
    const result =
        currentStr.slice(0, 4 - inputYear.length) + inputYear;
    return Number(result);
}

function parseValue(
    value: string,
    mode: "date" | "datetime"
): Date | null {
    if (!value) return null;

    // DATE ONLY
    if (mode === "date") {
        const parts = value.split("-");

        if (parts.length !== 3) return null;

        const y = Number(parts[0]);
        const m = Number(parts[1]);
        const d = Number(parts[2]);

        return new Date(y, m - 1, d);
    }

    // DATETIME
    const date = new Date(value);

    return isNaN(date.getTime())
        ? null
        : date;
}

function formatDateOnly(date: Date) {
    const y = date.getFullYear();
    const m = String(
        date.getMonth() + 1
    ).padStart(2, "0");
    const d = String(
        date.getDate()
    ).padStart(2, "0");

    return `${y}-${m}-${d}`;
}

function serializeDate(
    date: Date,
    mode: "date" | "datetime",
    time: string
) {
    if (mode === "date") {
        return formatDateOnly(date);
    }

    const [h, m] = time.split(":");

    date.setHours(h ? Number(h) : 0);
    date.setMinutes(m ? Number(m) : 0);

    return date.toISOString();
}
