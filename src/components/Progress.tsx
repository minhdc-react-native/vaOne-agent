import "../style/Progress.css";

interface ProgressProps {
    value: number;
    total: number;
}

export default function Progress({
    value,
    total,
}: ProgressProps) {
    const safeTotal = Math.max(total, 1);
    const percent = Math.min(
        100,
        Math.max(0, (value / safeTotal) * 100)
    );

    return (
        <div className="app-progress">
            <div className="app-progress-track">
                <div
                    className="app-progress-bar"
                    style={{ width: `${percent}%` }}
                />

                <div className="app-progress-label">
                    {Math.round(percent)}% ({value}/{total})
                </div>
            </div>
        </div>
    );
}