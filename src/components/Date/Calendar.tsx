import React, { useState } from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import clsx from "clsx";


interface CalendarProps {
    value?: Date | null;
    onSelect: (d: Date) => void;
}

const Calendar: React.FC<CalendarProps> = ({ value, onSelect }) => {
    const today = new Date();
    const daysOfWeek = [
        "CN",
        "T2",
        "T3",
        "T4",
        "T5",
        "T6",
        "T7"
    ];
    const monthNames = [
        "Tháng 1",
        "Tháng 2",
        "Tháng 3",
        "Tháng 4",
        "Tháng 5",
        "Tháng 6",
        "Tháng 7",
        "Tháng 8",
        "Tháng 9",
        "Tháng 10",
        "Tháng 11",
        "Tháng 12"
    ];

    const [displayMonth, setDisplayMonth] = useState<Date>(
        value ?? today
    );

    const year = displayMonth.getFullYear();
    const month = displayMonth.getMonth();

    const firstDayOfMonth = new Date(year, month, 1).getDay();
    const daysInMonth = new Date(year, month + 1, 0).getDate();

    const changeMonth = (delta: number) => {
        setDisplayMonth(
            new Date(year, month + delta, 1)
        );
    };

    const calendarCells: React.ReactNode[] = [];

    // empty cells before first day
    for (let i = 0; i < firstDayOfMonth; i++) {
        calendarCells.push(<div key={`empty-${i}`} />);
    }

    // days
    for (let day = 1; day <= daysInMonth; day++) {
        const isToday =
            today.getDate() === day &&
            today.getMonth() === month &&
            today.getFullYear() === year;

        const isSelected =
            value &&
            value.getDate() === day &&
            value.getMonth() === month &&
            value.getFullYear() === year;

        calendarCells.push(
            <div
                key={day}
                onClick={() => onSelect(new Date(year, month, day))}
                className={clsx('py-1 cursor-pointer rounded-full text-sm text-center',
                    isSelected && "bg-red-500 text-white",
                    !isSelected && isToday && "bg-red-100 text-red-600",
                    !isSelected && !isToday && "hover:bg-gray-200")}
            >
                {day}
            </div>
        );
    }

    return (
        <div className="p-3 w-62.5 select-none">
            {/* header */}
            <div className="flex items-center justify-between mb-3">
                <div
                    onClick={() => changeMonth(-1)}
                    className="p-1 rounded-full hover:bg-gray-100 hover:cursor-pointer"
                >
                    <ChevronLeft size={16} />
                </div>

                <div className="font-semibold text-sm">
                    {(monthNames as string[])[month]} {year}
                </div>

                <div
                    onClick={() => changeMonth(1)}
                    className="p-1 rounded-full hover:bg-gray-100 hover:cursor-pointer"
                >
                    <ChevronRight size={16} />
                </div>
            </div>

            {/* days of week */}
            <div className="grid grid-cols-7 text-xs text-gray-500 mb-2 text-center">
                {(daysOfWeek as string[]).map(d => (
                    <div key={d} >
                        {d}
                    </div>
                ))}
            </div>
            {/* calendar */}
            <div className="grid grid-cols-7 gap-1 text-sm">
                {calendarCells}
            </div>
        </div>
    );
};

export default Calendar;
