import {
    createContext,
    useContext,
    useEffect,
    useRef,
    useState,
    type ReactNode,
} from "react";
import clsx from "clsx";
import { IconLucide, IconName } from "./IconLucide";
import { Badge } from "./Badge";

type TabsOrientation = "horizontal" | "vertical";
type TabsVariant = "line" | "container" | "ghost";

type TabsContextType = {
    changeTab?: number;
    activeIndex: number;
    setActiveIndex: (i: number) => void;
    orientation: TabsOrientation;
    variant: TabsVariant;
    tabWidth?: number;
    lazy: boolean;
};

const TabsContext = createContext<TabsContextType | null>(null);

const useTabs = () => {
    const ctx = useContext(TabsContext);

    if (!ctx) {
        throw new Error("Tabs components must be inside <Tabs />");
    }

    return ctx;
};

const TabsRoot = ({
    children,
    orientation = "horizontal",
    variant = "container",
    defaultActiveTab = 0,
    activeTab,
    onChange,
    tabWidth,
    changeTab,
    lazy = false,
}: {
    children: ReactNode;
    orientation?: TabsOrientation;
    variant?: TabsVariant;
    defaultActiveTab?: number;
    activeTab?: number;
    changeTab?: number;
    onChange?: (index: number) => void;
    tabWidth?: number;
    lazy?: boolean;
}) => {
    const isControlled = activeTab !== undefined;

    const [internalIndex, setInternalIndex] = useState(defaultActiveTab);

    const currentIndex = isControlled
        ? activeTab
        : internalIndex;

    const setActiveIndex = (index: number) => {
        if (!isControlled) {
            setInternalIndex(index);
        }

        onChange?.(index);
    };

    useEffect(() => {
        setInternalIndex(defaultActiveTab);
    }, [defaultActiveTab]);

    return (
        <TabsContext.Provider
            value={{
                activeIndex: currentIndex,
                setActiveIndex,
                orientation,
                variant,
                tabWidth,
                changeTab,
                lazy
            }}
        >
            <div
                className={clsx(
                    "h-full min-h-0 w-full min-w-0",
                    orientation === "horizontal"
                        ? "flex flex-col"
                        : "flex flex-row"
                )}
            >
                {children}
            </div>
        </TabsContext.Provider>
    );
};

const TabsHeader = ({
    children,
    className,
}: {
    children: ReactNode;
    className?: string;
}) => {
    const { orientation } = useTabs();

    return (
        <div
            className={clsx(
                orientation === "horizontal"
                    ? "flex items-center px-1"
                    : "flex flex-col py-1",
                className
            )}
        >
            {children}
        </div>
    );
};

const Tab = ({
    children,
    index,
    icon,
    disabled,
    badge = null,
    onClose,
    labelClass,
    labelClassActive,
}: {
    children: ReactNode;
    index: number;
    icon?: IconName;
    disabled?: boolean;
    badge?: number | string | null;
    onClose?: (index: number) => void;
    labelClass?: string;
    labelClassActive?: string;
}) => {

    const {
        activeIndex,
        setActiveIndex,
        orientation,
        variant,
        tabWidth,
    } = useTabs();

    const isActive = activeIndex === index;

    const isContainer = variant === "container";
    const isGhost = variant === "ghost";
    const isHorizontal = orientation === "horizontal";

    const tabClass = clsx(
        // Base
        "relative flex items-center gap-2 text-sm",
        "transition-colors focus-danger",

        // Orientation
        isHorizontal
            ? "mt-1 px-4 py-1"
            : "mb-0 px-4 py-1",

        // Container
        isContainer &&
        (isHorizontal
            ? "rounded-t-sm border-l border-t border-r border-gray-300"
            : "rounded-l-sm border-l border-t border-b border-gray-300"),

        // Ghost
        isGhost &&
        (isHorizontal ? "rounded-t-sm border-t border-x"
            : "rounded-l-sm border-l border-t border-b border-gray-300"),

        // Line
        variant === "line" &&
        "border border-transparent rounded-sm",

        // Color
        isContainer
            ? isActive
                ? "bg-red-700 text-white"
                : "bg-gray-50 text-gray-700 hover:bg-gray-100"
            : isGhost
                ? isActive
                    ? "border-gray-300 bg-white text-red-700 shadow-sm"
                    : "border-transparent text-gray-700 hover:bg-gray-50"
                : isActive
                    ? "text-red-700"
                    : "text-gray-700 hover:text-red-600",

        // State
        disabled
            ? "cursor-not-allowed opacity-40"
            : "cursor-default",

        labelClass,
        isActive && labelClassActive
    );

    const iconClass = clsx(
        isActive && isContainer
            ? "text-white"
            : isActive
                ? "text-red-500"
                : ""
    );

    const closeClass = clsx(
        "rounded-full hover:cursor-pointer",
        isContainer
            ? isActive
                ? "border-white"
                : "border-gray-700"
            : isActive
                ? "border-white hover:bg-red-600 hover:text-white"
                : "border-gray-700 hover:bg-gray-500 hover:text-white"
    );

    return (
        <div
            onClick={() => {
                if (!disabled) {
                    setActiveIndex(index);
                }
            }}
            onKeyDown={(e) => {
                if (
                    (e.key === "Enter" || e.key === " ") &&
                    !disabled
                ) {
                    e.preventDefault();
                    setActiveIndex(index);
                }
            }}
            className={tabClass}
        >
            {icon && (
                <IconLucide
                    name={icon}
                    size={18}
                    className={iconClass}
                />
            )}

            <span
                className="block truncate"
                style={{ maxWidth: tabWidth }}
            >
                {children}
                <Badge
                    badge={badge}
                    variant={isContainer ? "outline" : "primary"}
                    active={isActive}
                />
            </span>

            {onClose && (
                <div className="absolute top-0 -right-1">
                    <IconLucide
                        name="X"
                        size={12}
                        className={closeClass}
                        onClick={(e: any) => {
                            e.stopPropagation();
                            onClose(index);
                        }}
                    />
                </div>
            )}

            {isActive && variant === "line" && isHorizontal && (
                <span className="absolute left-0 right-0 -bottom-0.5 h-0.5 rounded-full bg-red-700" />
            )}

            {isActive && variant === "line" && !isHorizontal && (
                <span className="absolute top-0 bottom-0 -right-0.5 w-0.5 bg-red-700" />
            )}
        </div>
    );
};

const TabsContent = ({
    children,
    className = "",
}: {
    children: ReactNode;
    className?: string;
}) => {
    return (
        <div
            className={clsx(
                "flex-1 overflow-hidden rounded-sm border border-gray-300",
                className
            )}
        >
            {children}
        </div>
    );
};

const Panel = ({
    children,
    index,
    className = "",
}: {
    children: ReactNode;
    index: number;
    className?: string;
}) => {
    const {
        activeIndex,
        changeTab,
        lazy,
    } = useTabs();

    const [mounted, setMounted] = useState(!lazy);

    const isChangeTab = changeTab === index;
    const isActive = activeIndex === index;

    const tabRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (isChangeTab) {
            tabRef.current?.focus();
        }
    }, [isChangeTab]);

    useEffect(() => {
        if (lazy && isActive) {
            // eslint-disable-next-line react-hooks/set-state-in-effect
            setMounted(true);
        }
    }, [lazy, isActive]);

    if (!mounted) {
        return null;
    }

    return (
        <div
            ref={tabRef}
            tabIndex={0}
            className={clsx(
                "focus-danger h-full rounded-b-sm bg-white px-2 py-1",
                !isActive && "hidden",
                className
            )}
        >
            {children}
        </div>
    );
};

export const Tabs = Object.assign(TabsRoot, {
    Header: TabsHeader,
    Tab,
    Content: TabsContent,
    Panel,
});