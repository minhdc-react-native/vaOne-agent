import clsx from "clsx";
interface IBadge {
    badge: number | string | null;
    className?: string,
    variant?: 'outline' | 'primary',
    active?: boolean
}
export const Badge = ({ badge, className, variant = "primary", active = true }: IBadge) => {
    if (!badge) return "";
    return (
        <span
            className={clsx(
                "absolute z-10 -top-0.5 -right-1 min-w-4 h-4 flex items-center justify-center text-[9px] rounded-full border",
                variant === "primary" ? `border-white ${active ? 'bg-red-500' : 'bg-gray-400'} text-white` :
                    `${active ? 'border-red-500' : 'border-gray-400'} bg-white ${active ? 'text-red-500' : 'text-gray-400'}`,
                className
            )}
        >
            {typeof badge === "number" && badge > 99 ? '+99' : badge}
        </span>
    );
}