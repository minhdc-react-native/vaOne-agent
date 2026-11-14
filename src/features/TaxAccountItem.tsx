import { Building2 } from "lucide-react";
import { ITaxAccount } from "../service/taxAccountService";

interface ITaxAccountItem {
    account?: ITaxAccount | null;
    handleSelect: (account: ITaxAccount) => void;
    index?: number;
    className?: string;
}
export const TaxAccountItem = ({ account, handleSelect, index, className }: ITaxAccountItem) => {
    if (!account) return null;
    return (
        <div
            key={account.tax_code}
            onClick={() => handleSelect(account)}
            className={`w-100 shrink-0 p-2 text-left} ${className}`}
        >
            <div className="flex items-start gap-3">
                <div className={`flex flex-col size-10 shrink-0
                items-center justify-center rounded
                `}>
                    <span className="text-xs">{(index ?? 0) + 1}</span>
                    <Building2 size={18} />
                </div>
                <div className="min-w-0 flex-1">
                    <div className="flex items-start
                                                        justify-between gap-2">
                        <div className="min-w-0 flex-1">
                            <div className="flex gap-1 items-center">
                                <p className="truncate
                                                                text-sm font-semibold">
                                    {account.tax_code}
                                </p>
                            </div>
                            <p className="mt-1 truncate
                                                                text-xs text-muted-foreground">
                                {account.invoice_type}
                            </p>
                        </div>

                    </div>
                    <span className="text-xs line-clamp-2 wrap-break-word">
                        {account.note}
                    </span>

                </div>
            </div>
        </div>
    );
}