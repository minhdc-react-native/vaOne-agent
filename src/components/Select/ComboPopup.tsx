import React, { forwardRef, useCallback, useImperativeHandle, useLayoutEffect, useMemo, useRef, useState } from "react";
import clsx from "clsx";
import { Placement } from "../../hook/useAutoPosition";
import { IconLucide } from "../IconLucide";
import PopupPosition from "../PopupPosition";
import Input from "../Input";
import { Line } from "../Line";
import { Tooltip } from "../Tooltip/Tooltip";

type IComboPopupProps<T> = {
    value?: T | null;
    data: T[];
    placement?: Placement;
    renderItem: (item?: T | null, isSelected?: boolean, index?: number) => React.ReactNode;
    renderDisplay?: (item?: T | null) => React.ReactNode;
    header?: React.ReactNode;
    addToolbar?: React.ReactNode;
    onSelect?: (item: T) => void;
    onClick?: (e: React.MouseEvent<HTMLDivElement, MouseEvent>, callBack: () => void) => Promise<void>;
    className?: string;
    showSearch?: boolean;
    haveBorder?: boolean;
    backdrop?: boolean;
    hiddenArrow?: boolean;
    zIndex?: number;
    isOption?: boolean;
    iconPosition?: 'left' | 'right' | 'none';
    disabled?: boolean;
    clearable?: boolean;
    onClear?: () => void;
    filedId?: string;
};
export interface ComboPopupRef {
    setOpen: (open: boolean) => void;
}
type ComboPopupComponent = <T>(
    props: IComboPopupProps<T> & {
        ref?: React.ForwardedRef<ComboPopupRef>;
    }
) => React.ReactElement | null;

const ComboPopupImpl = <T,>(
    {
        placement = 'bottom-end',
        value,
        data,
        onClick,
        onSelect,
        renderItem,
        renderDisplay,
        header,
        addToolbar,
        className,
        showSearch = false,
        haveBorder = true,
        hiddenArrow = false,
        backdrop,
        zIndex,
        isOption = false,
        iconPosition = 'none',
        disabled = false,
        onClear,
        filedId = "id"
    }: IComboPopupProps<T>,
    ref: React.ForwardedRef<ComboPopupRef>
) => {
    const divRef = useRef<any>(null);
    const [popupWidth, setPopupWidth] = useState<number>();
    const [open, setOpen] = useState(false);
    const [searchTerm, setSearchTerm] = useState("");
    const activeItemRef = useRef<HTMLDivElement | null>(null);

    const filteredData = useMemo(() => {
        return showSearch
            ? data.filter(item =>
                (item as any).label.toLowerCase().includes(searchTerm.toLowerCase())
            )
            : data
    }, [data, searchTerm, showSearch]);

    useLayoutEffect(() => {
        if (open && activeItemRef.current) {
            activeItemRef.current.scrollIntoView({
                block: "nearest",
            });
        }
        if (divRef.current) {
            setPopupWidth(divRef.current.offsetWidth);
        }
        console.log()
    }, [open]);

    useImperativeHandle(
        ref,
        () => ({
            setOpen: (value: boolean) => {
                setOpen(value);
            },
        }),
        []
    );
    const onClickCombo = useCallback((e: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
        e.stopPropagation();
        if (onClick) {
            onClick(e, () => {
                setOpen(prev => !prev);
            })
        } else {
            setOpen(prev => !prev);
        }
    }, [onClick]);

    return (
        <>
            <div ref={divRef}
                className={`flex items-center ${haveBorder ? "border border-(--border) hover:bg-gray-100" : ""} hover:cursor-pointer rounded-sm ${className}`}
                onClick={onClickCombo}>
                <div className="flex-1 min-w-0">
                    {renderDisplay ? renderDisplay(value) : renderItem(value)}
                </div>
                {onClear && value != null && !disabled && (
                    <Tooltip
                        title="Xóa lựa chọn"
                        onClick={(e) => {
                            e.stopPropagation();
                            onClear();
                        }}
                    >
                        <IconLucide name="X" size={14} className="text-red-500" />
                    </Tooltip>
                )}
                {!hiddenArrow && <IconLucide name="ChevronDown" size={18} className="mx-1 text-gray-500" />}
            </div>
            <PopupPosition
                dropdownRef={divRef as React.RefObject<HTMLElement>}
                isOpen={open} placement={placement} zIndex={zIndex}
                onClose={() => setOpen(false)} backdrop={backdrop}
                className="combobox-popup"
            >
                {(showSearch || addToolbar) && (
                    <div className="p-2 flex justify-between border-b border-gray-300 gap-1 items-center">
                        {addToolbar || <div />}
                        {showSearch && <Input autoFocus value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} placeholder="Tìm kiếm..." />}
                    </div>
                )}
                <div className="max-h-100 overflow-y-auto flex flex-col min-w-0">
                    {header}
                    {filteredData.length === 0 && (
                        <div className="p-5 text-gray-500 italic">Không có kết quả</div>
                    )}
                    {filteredData.map((item, idx) => {
                        const isActive =
                            (value as any)?.[filedId] === (item as any)[filedId];
                        return (
                            <React.Fragment key={(item as any)[filedId]}>
                                <div
                                    ref={isActive ? activeItemRef : null}
                                    className={clsx("flex min-w-0 items-center gap-2 p-1 justify-between hover:cursor-pointer hover:bg-gray-100",
                                        (value as any)?.[filedId] === (item as any)[filedId] ? "bg-amber-50" : ""
                                    )}
                                    key={(item as any)[filedId]}
                                    onClick={(e) => {
                                        setOpen(false);
                                        if (!disabled) onSelect?.(item);
                                    }}
                                >
                                    {iconPosition === "left" && <IconLucide name={isOption ? (isActive ? "CircleCheckBig" : "Circle") : "Check"} size={16} className={clsx("ml-1", isActive ? "text-red-500" : (isOption ? "text-gray-300" : "text-transparent"))} />}
                                    <div className="flex-1 min-w-0">
                                        {renderItem(item, isActive, idx)}
                                    </div>
                                    {iconPosition === "right" && <IconLucide name={isOption ? (isActive ? "CircleCheckBig" : "Circle") : "Check"} size={16} className={clsx("mr-1", isActive ? "text-red-500" : (isOption ? "text-gray-300" : "text-transparent"))} />}
                                </div>
                                {idx !== filteredData.length - 1 && (
                                    <div className="h-px w-full shrink-0 bg-gray-300" />
                                )}
                            </React.Fragment>
                        );
                    })}
                </div>
            </PopupPosition>
        </>
    );
};

export const ComboPopup = forwardRef(
    ComboPopupImpl
) as ComboPopupComponent;