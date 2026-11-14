import { XMLParser } from "fast-xml-parser";
import type { IInvoice, IInvoiceItem, ITaxItem } from "./InvoicePreview";

const ARRAY_XML_PATHS = [
    "HDon.DLHDon.NDHDon.DSHHDVu.HHDVu",
    "HDon.DLHDon.TTChung.TTKhac.TTin",
    "HDon.DLHDon.NDHDon.TToan.THTTLTSuat.LTSuat",
];

const parser = new XMLParser({
    ignoreAttributes: false,
    attributeNamePrefix: "@_",
    parseTagValue: true,
    parseAttributeValue: true,
    trimValues: true,
    isArray: (_name, jpath) =>
        typeof jpath === "string" &&
        ARRAY_XML_PATHS.includes(jpath),
});

export function parseXmlToJson(
    xml: string | null
): any {
    if (!xml?.trim()) {
        return {};
    }
    try {
        return parser.parse(xml);
    } catch (error) {
        return {}
    }
}

export function parseInvoiceXml(
    xml: string
): IInvoice | null {
    if (!xml?.trim()) {
        return null;
    }

    try {
        const data = parser.parse(xml);

        const hDon = data?.HDon;
        const dlHDon = hDon?.DLHDon;

        if (!dlHDon) {
            return null;
        }

        const ttChung = dlHDon?.TTChung;
        const ndHDon = dlHDon?.NDHDon;

        const nBan = ndHDon?.NBan;
        const nMua = ndHDon?.NMua;

        return {
            general: {
                version: toStringValue(
                    ttChung?.PBan
                ),

                invoiceName: toStringValue(
                    ttChung?.THDon
                ),

                invoiceForm: toStringValue(
                    ttChung?.KHMSHDon
                ),

                invoiceSeries: toStringValue(
                    ttChung?.KHHDon
                ),

                invoiceNumber: toStringValue(
                    ttChung?.SHDon
                ),

                issueDate: toStringValue(
                    ttChung?.NLap
                ),

                currency: toStringValue(
                    ttChung?.DVTTe
                ),

                exchangeRate: toNumber(
                    ttChung?.TGia
                ),

                paymentMethod: toStringValue(
                    ttChung?.HTTToan
                ),

                taxAuthorityCode:
                    toStringValue(
                        hDon?.MCCQT
                    ),
            },

            seller: {
                name: toStringValue(
                    nBan?.Ten
                ),

                taxCode: toStringValue(
                    nBan?.MST
                ),

                address: toStringValue(
                    nBan?.DChi
                ),

                phone: toStringValue(
                    nBan?.SDThoai
                ),
            },

            buyer: {
                name: toStringValue(
                    nMua?.Ten
                ),

                taxCode: toStringValue(
                    nMua?.MST
                ),

                address: toStringValue(
                    nMua?.DChi
                ),
            },

            items: parseInvoiceItems(
                ndHDon?.DSHHDVu?.HHDVu
            ),
            taxItems: parseTaxItems(ndHDon?.TToan?.THTTLTSuat?.LTSuat),
            payment: {
                totalBeforeTax: toNumber(
                    ndHDon?.TToan?.TgTCThue
                ),

                totalDiscount: toNumber(
                    ndHDon?.TToan?.TTCKTMai
                ),

                totalTax: toNumber(
                    ndHDon?.TToan?.TgTThue
                ),

                totalPayment: toNumber(
                    ndHDon?.TToan?.TgTTTBSo
                ),

                totalPaymentInWords:
                    toStringValue(
                        ndHDon?.TToan?.TgTTTBChu
                    ),
            },

            qrCode: toStringValue(
                hDon?.DLQRCode
            ),

            taxAuthorityCode:
                toStringValue(hDon?.MCCQT),
        };
    } catch (error) {
        console.error(
            "Không thể parse XML hóa đơn:",
            error
        );

        return null;
    }
}


function parseInvoiceItems(
    value: unknown
): IInvoiceItem[] {
    if (!value) {
        return [];
    }

    const items = Array.isArray(value)
        ? value
        : [value];

    return items.map(
        (item: any, index: number) => ({
            index:
                toNumber(item?.STT) ??
                index + 1,

            code: toStringValue(
                item?.MHHDVu
            ),

            name: toStringValue(
                item?.THHDVu
            ),

            unit: toStringValue(
                item?.DVTinh
            ),

            quantity: toNumber(
                item?.SLuong
            ),

            unitPrice: toNumber(
                item?.DGia
            ),

            discountRate: toNumber(
                item?.TLCKhau
            ),

            discountAmount: toNumber(
                item?.STCKhau
            ),

            amount: toNumber(
                item?.ThTien
            ),

            taxRate: toStringValue(
                item?.TSuat
            ),
        })
    );
}

function parseTaxItems(
    value: unknown
): ITaxItem[] {
    if (!value) {
        return [];
    }

    const items = Array.isArray(value)
        ? value
        : [value];

    return items.map(
        (item: any, index: number) => ({
            taxRate: toStringValue(
                item.TSuat
            ),
            amount: toNumber(
                item?.ThTien
            ),

            taxAmount: toNumber(
                item?.TThue
            ),
        })
    );
}


function toNumber(
    value: unknown
): number | undefined {
    if (
        value === undefined ||
        value === null ||
        value === ""
    ) {
        return undefined;
    }

    const result = Number(value);

    return Number.isNaN(result)
        ? undefined
        : result;
}

function toStringValue(
    value: unknown
): string | undefined {
    if (
        value === undefined ||
        value === null
    ) {
        return undefined;
    }

    return String(value).trim();
}