import clsx from "clsx";

interface ILoading {
    className?: string;
}
export function Loading({ className = 'h-8 w-8' }: ILoading) {
    return (
        <div className={clsx("animate-spin rounded-full border-2 border-black/30 border-t-(--primary)", className)} />
    );
}