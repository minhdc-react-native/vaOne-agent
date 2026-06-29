import { useLoading } from "../service/loading.service";

export function GlobalLoading() {
    const { state } = useLoading();

    if (!state.visible) return null;

    return (
        <div className="fixed inset-0 z-9999 flex flex-col items-center justify-center bg-black/40 backdrop-blur-sm">
            {/* Spinner */}
            <div className="h-10 w-10 animate-spin rounded-full border-4 border-white/30 border-t-white" />

            {/* Text */}
            {state.text && (
                <div className="mt-3 text-sm text-white">
                    {state.text}
                </div>
            )}
        </div>
    );
}