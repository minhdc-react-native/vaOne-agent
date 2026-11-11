import { IPdfProgress } from "../pages/PreviewReport";
import Progress from "../components/Progress";
export interface IInvoice {
    general: {
        version?: string;
        invoiceName?: string;
        invoiceForm?: string;
        invoiceSeries?: string;
        invoiceNumber?: string;
        issueDate?: string;
        currency?: string;
        exchangeRate?: number;
        paymentMethod?: string;
        taxAuthorityCode?: string;
    };

    seller: {
        name?: string;
        taxCode?: string;
        address?: string;
        phone?: string;
    };

    buyer: {
        name?: string;
        taxCode?: string;
        address?: string;
    };

    items: IInvoiceItem[];
    taxItems: ITaxItem[],
    payment: {
        totalBeforeTax?: number;
        totalDiscount?: number;
        totalTax?: number;
        totalPayment?: number;
        totalPaymentInWords?: string;
    };

    qrCode?: string;
    taxAuthorityCode?: string;
}

export interface IInvoiceItem {
    index: number;
    code?: string;
    name?: string;
    unit?: string;
    quantity?: number;
    unitPrice?: number;
    discountRate?: number;
    discountAmount?: number;
    amount?: number;
    taxRate?: string;
}

export interface ITaxItem {
    taxRate?: string;
    amount?: number;
    taxAmount?: number;
}

interface InvoicePreviewProps {
    urlPdf?: string | null;
    progress: IPdfProgress
}

export function InvoicePreview({
    urlPdf,
    progress
}: InvoicePreviewProps) {
    return (
        <div className="min-w-fit h-full">
            {urlPdf ? <iframe
                src={urlPdf}
                className="w-full h-full"
            /> : <div className="flex flex-1 items-center justify-center">
                <div className="w-100  h-100 flex flex-col gap-2 justify-center">
                    <Progress
                        message={progress.message}
                        value={progress.current}
                        total={progress.total}
                    />
                </div>
            </div>}
        </div>
    );
}
