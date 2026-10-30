
import { Loader2 } from "lucide-react";

interface ModalOverlayProps {
    open: boolean;
}

export default function ModalOverlay({ open }: ModalOverlayProps) {
    if (!open) return null;

    return (
        <div className="fixed inset-0 z-[99999] flex items-center justify-center bg-black/30 backdrop-blur-[2px]" />
    );
}
