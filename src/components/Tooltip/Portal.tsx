import { clsx } from "clsx";
import { createPortal } from "react-dom";
interface IProgs {
    children: React.ReactNode,
    zIndex?: number
}
export function Portal({ children, zIndex = 99 }: IProgs) {
    return createPortal(
        <div className={clsx(`fixed pointer-events-none`)} style={{ zIndex }}>
            {children}
        </div>,
        document.body
    );
}
