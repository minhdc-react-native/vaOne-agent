
import { useMemo, useState } from "react";
import {
    Copy,
    Check,
    Braces,
    CodeXml,
    File,
} from "lucide-react";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark, oneLight } from "react-syntax-highlighter/dist/esm/styles/prism";
import { ISyncInvoice } from "../service/syncInvoicesService";
import { InvoicePreview } from "./InvoicePreview";
import clsx from "clsx";

type DataTab = "view" | "json" | "xml";

interface IInvoiceDataViewer {
    invoice: ISyncInvoice;
    className?: string;
}

export function InvoiceDataViewer({
    invoice,
    className,
}: IInvoiceDataViewer) {
    const [activeTab, setActiveTab] = useState<DataTab>("view");
    const [copied, setCopied] = useState(false);
    const [expanded, setExpanded] = useState(false);

    const jsonContent = useMemo(() => {
        if (invoice.invoice == null) return "";

        try {
            const value = JSON.parse(invoice.invoice);

            return JSON.stringify(value, null, 2);
        } catch {
            return invoice.invoice;
        }
    }, [invoice.invoice]);

    const xmlContent = useMemo(() => {
        if (!invoice.xml_data) return "";

        return formatXml(invoice.xml_data);
    }, [invoice.xml_data]);

    const content = activeTab === "json"
        ? jsonContent
        : xmlContent;

    const lineCount = content
        ? content.split("\n").length
        : 0;

    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(content);
            setCopied(true);
            window.setTimeout(() => setCopied(false), 1500);
        } catch (error) {
            console.error("Copy failed:", error);
        }
    };



    const hasContent = Boolean(content.trim());

    return (
        <div
            className={[
                "flex flex-col overflow-hidden rounded-lg border border-gray-200 bg-white",
                className,
            ].filter(Boolean).join(" ")}
        >
            {/* Tabs */}
            <div className="flex items-center border-b border-gray-200 px-3">
                <button
                    type="button"
                    onClick={() => {
                        setActiveTab("view");
                        setCopied(false);
                    }}
                    className={[
                        "flex items-center gap-2 border-b-2 px-4 py-3 text-sm font-medium transition-colors",
                        activeTab === "view"
                            ? "border-primary text-primary"
                            : "border-transparent text-gray-500 hover:text-gray-800",
                    ].join(" ")}
                >
                    <File size={16} />
                    Thông tin
                </button>
                <button
                    type="button"
                    onClick={() => {
                        setActiveTab("xml");
                        setCopied(false);
                    }}
                    className={[
                        "flex items-center gap-2 border-b-2 px-4 py-3 text-sm font-medium transition-colors",
                        activeTab === "xml"
                            ? "border-primary text-primary"
                            : "border-transparent text-gray-500 hover:text-gray-800",
                    ].join(" ")}
                >
                    <CodeXml size={16} />
                    XML
                </button>
                <button
                    type="button"
                    onClick={() => {
                        setActiveTab("json");
                        setCopied(false);
                    }}
                    className={[
                        "flex items-center gap-2 border-b-2 px-4 py-3 text-sm font-medium transition-colors",
                        activeTab === "json"
                            ? "border-primary text-primary"
                            : "border-transparent text-gray-500 hover:text-gray-800",
                    ].join(" ")}
                >
                    <Braces size={16} />
                    JSON
                </button>
            </div>

            {/* Content */}
            <div
                className={[
                    "min-h-0 overflow-auto bg-gray-200",
                    expanded ? "h-[calc(100vh-180px)]" : "h-125",
                ].join(" ")}
            >
                {activeTab === "view" ? (
                    invoice.xml_data ? (
                        <InvoicePreview xml={invoice.xml_data} />
                    ) : (
                        <div className="flex h-full min-h-75 flex-col items-center justify-center gap-2 text-gray-900">
                            <File size={32} />
                            <span className="text-sm">Không có dữ liệu hóa đơn</span>
                        </div>
                    )
                ) : hasContent ? (
                    <SyntaxHighlighter
                        language={activeTab === "json" ? "json" : "xml"}
                        style={oneLight}
                        showLineNumbers
                        wrapLongLines={false}
                        customStyle={{
                            margin: 0,
                            padding: "16px 0",
                            background: "#e5e7eb",
                            fontSize: "12px",
                            lineHeight: "1.7",
                            minHeight: "100%",
                        }}
                        lineNumberStyle={{
                            minWidth: "48px",
                            paddingRight: "16px",
                            color: "#6b7280",
                            textAlign: "right",
                            userSelect: "none",
                        }}
                        codeTagProps={{
                            style: {
                                fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
                            },
                        }}
                    >
                        {content}
                    </SyntaxHighlighter>
                ) : (
                    <div className="flex h-full min-h-75 flex-col items-center justify-center gap-2 text-gray-900">
                        {activeTab === "json" ? <Braces size={32} /> : <CodeXml size={32} />}
                        <span className="text-sm">
                            Không có dữ liệu {activeTab.toUpperCase()}
                        </span>
                    </div>
                )}
            </div>

            {/* Footer */}
            <div className="flex items-center justify-between border-t border-gray-200 bg-gray-50 px-4 py-2">
                {activeTab !== "view" ? <div className="flex items-center gap-3 text-[11px] text-gray-500">
                    <span>{lineCount} dòng</span>
                    <span>UTF-8</span>
                    <span>{activeTab.toUpperCase()}</span>
                </div> : <div className="flex items-center gap-3 text-[11px] text-gray-500">Hiển thị từ dữ liệu XML</div>}

                <button
                    type="button"
                    onClick={handleCopy}
                    disabled={!hasContent || activeTab !== "view"}
                    className={clsx("flex items-center cursor-pointer gap-1.5 rounded-md border border-gray-200 px-2.5 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-50", activeTab === "view" && "opacity-0")}
                >
                    {copied ? (
                        <Check size={14} className="text-green-600" />
                    ) : (
                        <Copy size={14} />
                    )}
                    {copied ? "Đã copy" : "Sao chép"}
                </button>
            </div>
        </div>
    );
}

const formatXml = (xml: string): string => {
    try {
        const parser = new DOMParser();
        const doc = parser.parseFromString(xml, "application/xml");

        const parserError = doc.querySelector("parsererror");

        if (parserError) {
            return xml;
        }

        const formatNode = (node: Node, level = 0): string => {
            const indent = "  ".repeat(level);
            const childIndent = "  ".repeat(level + 1);

            // Text node
            if (node.nodeType === Node.TEXT_NODE) {
                return node.textContent?.trim() || "";
            }

            // Comment
            if (node.nodeType === Node.COMMENT_NODE) {
                return `${indent}<!--${node.textContent}-->`;
            }

            if (node.nodeType !== Node.ELEMENT_NODE) {
                return "";
            }

            const element = node as Element;

            // Attributes
            const attributes = Array.from(element.attributes)
                .map(
                    (attr) =>
                        ` ${attr.name}="${attr.value}"`
                )
                .join("");

            const children = Array.from(element.childNodes)
                .filter((child) => {
                    if (child.nodeType === Node.TEXT_NODE) {
                        return child.textContent?.trim();
                    }

                    return true;
                });

            // Empty element
            if (children.length === 0) {
                return `${indent}<${element.tagName}${attributes}/>`;
            }

            // Chỉ có text
            const hasElementChild = children.some(
                (child) => child.nodeType === Node.ELEMENT_NODE
            );

            if (!hasElementChild) {
                const text = children
                    .map((child) => child.textContent ?? "")
                    .join("")
                    .trim();

                return `${indent}<${element.tagName}${attributes}>${text}</${element.tagName}>`;
            }

            // Có child element
            const childContent = children
                .map((child) => formatNode(child, level + 1))
                .filter(Boolean)
                .join("\n");

            return [
                `${indent}<${element.tagName}${attributes}>`,
                childContent,
                `${indent}</${element.tagName}>`,
            ].join("\n");
        };

        return formatNode(doc.documentElement);
    } catch {
        return xml;
    }
};