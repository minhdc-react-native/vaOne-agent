import React, { useMemo } from "react";
import { parseInvoiceXml } from "./parseInvoiceXml";
import { QRCodeSVG } from "qrcode.react";
import clsx from "clsx";

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
    xml: string;
}

export function InvoicePreview({
    xml,
}: InvoicePreviewProps) {
    const invoice = useMemo(
        () => parseInvoiceXml(xml),
        [xml]
    );

    if (!invoice) {
        return (
            <div className="min-w-fit p-4">
                <div className="mx-auto min-h-[297mm] w-[210mm] bg-white p-[10mm] shadow-md">
                    XML hóa đơn không hợp lệ
                </div>
            </div>
        );
    }

    return (
        <div className="min-w-fit p-4">
            <div className="mx-auto min-h-[297mm] w-[210mm] bg-white p-[10mm] shadow-md">
                <InvoiceHeader
                    general={invoice.general} qrCode={invoice.qrCode} taxAuthorityCode={invoice.taxAuthorityCode}
                />

                <InvoiceSeller
                    seller={invoice.seller}
                />

                <InvoiceBuyer
                    buyer={invoice.buyer}
                    general={invoice.general}
                />

                <InvoiceItems
                    items={invoice.items}
                />

                <InvoicePayment
                    payment={invoice.payment} taxItems={invoice.taxItems}
                />

                <InvoiceFooter
                    invoice={invoice}
                />
            </div>
        </div>
    );
}


interface InvoiceHeaderProps {
    general: IInvoice["general"];
    qrCode?: string;
    taxAuthorityCode?: string;
}

function InvoiceHeader({
    general,
    qrCode,
    taxAuthorityCode
}: InvoiceHeaderProps) {
    return (
        <>
            <div className="flex gap-2">
                {/* Logo */}
                <div className="flex justify-center">
                    {qrCode ? (
                        <QRCodeSVG
                            value={qrCode}
                            size={90}
                            level="M"
                            marginSize={0}
                        />
                    ) : (
                        <div className="flex h-22.5 w-22.5 items-center justify-center border border-dashed border-gray-300 text-[10px] text-gray-400">
                            Không có QR
                        </div>
                    )}
                </div>

                {/* Title */}
                <div className="flex-1 text-center">
                    <div className="text-sm font-bold uppercase">
                        {general.invoiceName ||
                            "HÓA ĐƠN GIÁ TRỊ GIA TĂNG"}
                    </div>

                    {/* <div className="mt-1 text-xs italic">
                        (Electronic VAT Invoice)
                    </div> */}
                    <div className="mt-1 text-center text-xs italic">
                        Ngày {formatDateVietnamese(general.issueDate)}
                    </div>
                    {taxAuthorityCode && (
                        <div className="mt-1 text-center text-xs">
                            <span className="font-semibold">
                                Mã CQT:
                            </span>{" "}
                            {taxAuthorityCode}
                        </div>
                    )}
                </div>

                {/* Invoice information */}
                <div className="text-xs leading-6">
                    <div>
                        <span className="font-bold">
                            Mẫu số:
                        </span>{" "}
                        {general.invoiceForm}
                    </div>

                    <div>
                        <span className="font-bold">
                            Ký hiệu:
                        </span>{" "}
                        {general.invoiceSeries}
                    </div>

                    <div>
                        <span className="font-bold">
                            Số:
                        </span>{" "}
                        <span className="font-bold">
                            {general.invoiceNumber}
                        </span>
                    </div>
                </div>
            </div>
        </>
    );
}

interface InvoiceSellerProps {
    seller: IInvoice["seller"];
}

function InvoiceSeller({
    seller,
}: InvoiceSellerProps) {
    return (
        <div className="mt-4 space-y-1 text-xs">
            <InvoiceInfoRow
                label="Đơn vị bán hàng"
                value={seller.name}
            />

            <InvoiceInfoRow
                label="Mã số thuế"
                value={seller.taxCode}
            />

            <InvoiceInfoRow
                label="Địa chỉ"
                value={seller.address}
            />

            {seller.phone && (
                <InvoiceInfoRow
                    label="Điện thoại"
                    value={seller.phone}
                />
            )}
        </div>
    );
}

interface InvoiceInfoRowProps {
    label: string;
    value?: React.ReactNode;
    className?: string;
}

function InvoiceInfoRow({
    label,
    value,
    className
}: InvoiceInfoRowProps) {
    return (
        <div className={clsx("flex text-xs", className)}>
            <span className="mr-1 font-semibold">
                {label}:
            </span>

            <span className="min-w-0">
                {value || ""}
            </span>
        </div>
    );
}

interface InvoiceBuyerProps {
    buyer: IInvoice["buyer"];
    general: IInvoice["general"];
}

function InvoiceBuyer({
    buyer,
    general,
}: InvoiceBuyerProps) {
    return (
        <div className="mt-3 space-y-1 text-xs">
            <InvoiceInfoRow
                label="Họ tên người mua hàng"
                value={buyer.name}
            />

            <InvoiceInfoRow
                label="Tên đơn vị"
                value={buyer.name}
            />

            <InvoiceInfoRow
                label="Mã số thuế"
                value={buyer.taxCode}
            />

            <InvoiceInfoRow
                label="Địa chỉ"
                value={buyer.address}
            />

            <div className="grid grid-cols-2 gap-x-8">
                <InvoiceInfoRow
                    label="Hình thức thanh toán"
                    value={general.paymentMethod}
                />

                <InvoiceInfoRow
                    label="Đồng tiền"
                    value={general.currency}
                    className="justify-end"
                />
            </div>
        </div>
    );
}

interface InvoiceItemsProps {
    items: IInvoiceItem[];
}

function InvoiceItems({
    items,
}: InvoiceItemsProps) {
    return (
        <table className="mt-4 w-full border-collapse border border-black text-[12px]">
            <thead>
                <tr className="font-semibold">
                    <th className="w-[38px] border border-black px-2 py-1 text-center">
                        STT
                    </th>

                    <th className="w-[90px] border border-black px-2 py-1 text-center">
                        Mã hàng hóa
                    </th>

                    <th className="border border-black px-2 py-1 text-center">
                        Tên hàng hóa, dịch vụ
                    </th>

                    <th className="w-[55px] border border-black px-2 py-1 text-center">
                        ĐVT
                    </th>

                    <th className="w-[75px] border border-black px-2 py-1 text-center">
                        Số lượng
                    </th>

                    <th className="w-[90px] border border-black px-2 py-1 text-center">
                        Đơn giá
                    </th>

                    <th className="w-[100px] border border-black px-2 py-1 text-center">
                        Thành tiền
                    </th>

                    <th className="w-[65px] border border-black px-2 py-1 text-center">
                        Thuế suất
                    </th>
                </tr>
            </thead>

            <tbody>
                {items.map((item) => (
                    <tr key={item.index}>
                        <td className="border border-black px-2 py-1 text-center align-top">
                            {item.index}
                        </td>

                        <td className="border border-black px-2 py-1 align-top">
                            {item.code}
                        </td>

                        <td className="border border-black px-2 py-1 align-top">
                            {item.name}
                        </td>

                        <td className="border border-black px-2 py-1 text-center align-top">
                            {item.unit}
                        </td>

                        <td className="border border-black px-2 py-1 text-right align-top">
                            {formatNumber(item.quantity)}
                        </td>

                        <td className="border border-black px-2 py-1 text-right align-top">
                            {formatMoney(item.unitPrice)}
                        </td>

                        <td className="border border-black px-2 py-1 text-right align-top">
                            {formatMoney(item.amount)}
                        </td>

                        <td className="border border-black px-2 py-1 text-center align-top">
                            {item.taxRate}
                        </td>
                    </tr>
                ))}

                {items.length === 0 && (
                    <tr>
                        <td
                            colSpan={8}
                            className="border border-black py-6 text-center"
                        >
                            Không có hàng hóa, dịch vụ
                        </td>
                    </tr>
                )}
            </tbody>
        </table>
    );
}
interface TaxItemsProps {
    items: ITaxItem[];
}
function TaxItems({
    items,
}: TaxItemsProps) {
    return (
        <table className="mt-2 w-full border-collapse border border-black text-xs">
            <thead>
                <tr className="font-semibold">
                    <th className="w-16.25 border border-black px-2 py-1 text-center">
                        Thuế suất
                    </th>

                    <th className="w-25 border border-black px-2 py-1 text-center">
                        Thành tiền
                    </th>
                    <th className="w-25 border border-black px-2 py-1 text-center">
                        Tiền thuế
                    </th>
                </tr>
            </thead>

            <tbody>
                {items.map((item, index) => (
                    <tr key={`taxItem-${index}`}>
                        <td className="border border-black px-2 py-1 text-center align-top">
                            {item.taxRate}
                        </td>
                        <td className="border border-black px-2 py-1 text-right align-top">
                            {formatMoney(item.amount)}
                        </td>
                        <td className="border border-black px-2 py-1 text-right align-top">
                            {formatMoney(item.taxAmount)}
                        </td>
                    </tr>
                ))}

                {items.length === 0 && (
                    <tr>
                        <td
                            colSpan={8}
                            className="border border-black py-6 text-center"
                        >
                            Không có hàng hóa, dịch vụ
                        </td>
                    </tr>
                )}
            </tbody>
        </table>
    );
}

interface InvoicePaymentProps {
    payment: IInvoice["payment"];
    taxItems: IInvoice["taxItems"];
}

function InvoicePayment({
    payment,
    taxItems
}: InvoicePaymentProps) {
    return (
        <div className="mt-2 flex text-xs gap-2 items-start">
            <div className="flex-1">
                <TaxItems items={taxItems} />
            </div>
            <div className="flex-1">
                <div className="grid grid-cols-[1fr_150px] py-1">
                    <div>
                        Tổng tiền hàng:
                    </div>

                    <div className="text-right">
                        {formatMoney(payment.totalBeforeTax)}
                    </div>
                </div>
                <div className="grid grid-cols-[1fr_150px] py-1">
                    <div>
                        Tổng tiền chiết khấu:
                    </div>

                    <div className="text-right">
                        {formatMoney(payment.totalDiscount)}
                    </div>
                </div>
                <div className="grid grid-cols-[1fr_150px] py-1">
                    <div>
                        Tổng tiền thuế:
                    </div>

                    <div className="text-right">
                        {formatMoney(payment.totalTax)}
                    </div>
                </div>

                <div className="grid grid-cols-[1fr_150px] py-1 font-bold">
                    <div>
                        Tổng cộng tiền thanh toán:
                    </div>

                    <div className="text-right">
                        {formatMoney(payment.totalPayment)}
                    </div>
                </div>

                <div className="mt-1">
                    <span className="font-bold">
                        Số tiền viết bằng chữ:
                    </span>{" "}
                    <span className="italic">
                        {payment.totalPaymentInWords}
                    </span>
                </div>
            </div>
        </div>
    );
}

interface InvoiceFooterProps {
    invoice: IInvoice;
}

function InvoiceFooter({
    invoice,
}: InvoiceFooterProps) {
    return (
        <div className="mt-8 text-xs">
            <div className="grid grid-cols-2 gap-8">

                {/* Người mua */}
                <div className="text-center">
                    <div className="font-bold">
                        NGƯỜI MUA HÀNG
                    </div>

                    <div className="mt-1 text-[11px] italic">
                        (Chữ ký số (nếu có))
                    </div>

                    <div className="h-24" />
                </div>
                {/* Người bán */}
                <div className="text-center">
                    <div className="font-bold">
                        NGƯỜI BÁN HÀNG
                    </div>

                    <div className="mt-1 text-[11px] italic">
                        (Chữ ký điện tử, chữ ký số)
                    </div>

                    <div className="h-24" />
                </div>
            </div>

            <div className="mt-4 text-gray-700 italic flex justify-center">
                (Cần kiểm tra, đối chiếu khi lập, nhận hóa đơn)
            </div>
        </div>
    );
}

function formatMoney(
    value?: number
): string {
    if (value === undefined || value === null) {
        return "";
    }

    return new Intl.NumberFormat("vi-VN", {
        maximumFractionDigits: 0,
    }).format(value);
}

function formatNumber(
    value?: number
): string {
    if (value === undefined || value === null) {
        return "";
    }

    return new Intl.NumberFormat("vi-VN", {
        maximumFractionDigits: 4,
    }).format(value);
}

function formatDateVietnamese(
    date?: string
): string {
    if (!date) {
        return "";
    }

    const [year, month, day] = date.split("-");

    if (!year || !month || !day) {
        return date;
    }

    return `${day} tháng ${month} năm ${year}`;
}