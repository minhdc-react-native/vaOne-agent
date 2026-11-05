
export const INVOICE_TYPES: { value: number, label: string, description?: string }[] = [
    {
        value: 1,
        label: "1. HĐ mua vào",
    },
    {
        value: 2,
        label: "2. HĐ bán ra",
    },
    {
        value: 3,
        label: "3. HĐ mua vào (MTT)",
    },
    {
        value: 4,
        label: "4. HĐ bán ra (MTT)",
    },
] as const;

export type InvoiceType = (typeof INVOICE_TYPES)[number]["value"];

import { useMemo } from "react";
import Checkbox from "../components/Checkbox";

interface InvoiceTypeCheckboxProps {
    value: string;
    onChange: (value: string) => void;
    disabled?: boolean;
}

export default function InvoiceTypeCheckbox({
    value,
    onChange,
    disabled = false,
}: InvoiceTypeCheckboxProps) {
    const selectedTypes = useMemo<InvoiceType[]>(() => {
        try {
            const parsed: unknown = JSON.parse(value || "[1,2,3,4]");

            if (!Array.isArray(parsed)) {
                return [];
            }

            return INVOICE_TYPES
                .filter((item) => parsed.includes(item.value))
                .map((item) => item.value);
        } catch {
            return [];
        }
    }, [value]);

    const handleChange = (
        invoiceType: InvoiceType,
        checked: boolean,
    ) => {
        const nextValue = checked
            ? [...new Set([...selectedTypes, invoiceType])]
            : selectedTypes.filter((item) => item !== invoiceType);

        const orderedValue = INVOICE_TYPES
            .filter((item) => nextValue.includes(item.value))
            .map((item) => item.value);

        onChange(JSON.stringify(orderedValue));
    };

    return (
        <div className="flex flex-col gap-3 p-1">
            <div className="text-xs font-medium border-b border-gray-300">
                Loại hóa đơn đồng bộ
            </div>

            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                {INVOICE_TYPES.map((item) => (
                    <Checkbox
                        key={item.value}
                        checked={selectedTypes.includes(item.value)}
                        onChange={(checked) =>
                            handleChange(item.value, checked)
                        }
                        label={item.label}
                        description={item.description}
                        disabled={disabled}
                    />
                ))}
            </div>
        </div>
    );
}