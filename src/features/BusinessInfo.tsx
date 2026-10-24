import { Building2, Hash } from "lucide-react";

interface BusinessInfoProps {
    id?: string | number | null;
    value?: string | null;
    tax_code?: string | null;
    className?: string;
}

export function BusinessInfo({
    id,
    value,
    tax_code,
    className,
}: BusinessInfoProps) {
    return (
        <div className={`flex min-w-0 flex-col gap-1 px-1 ${className ?? ""}`}>
            <div className="flex min-w-0 items-center gap-2">
                <Building2 className="size-4 shrink-0 text-muted-foreground" />

                <span className="truncate text-xs font-medium text-foreground">
                    {value || "Chưa có tên doanh nghiệp"}
                </span>

                {id != null && (
                    <span className="shrink-0 rounded border border-blue-200 bg-blue-50 px-1.5 py-0.5 text-[10px] font-medium text-blue-700">
                        {id}
                    </span>
                )}
            </div>

            <div className="flex items-center gap-1.5 pl-6 text-xs text-muted-foreground">
                <Hash className="size-3 shrink-0" />
                <span className="shrink-0">MST:</span>
                <span className="truncate text-xs text-foreground/80">
                    {tax_code || "Chưa có mã số thuế"}
                </span>
            </div>
        </div>
    );
}