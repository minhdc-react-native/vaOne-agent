import { Placement } from "../../hook/useAutoPosition";
import { Tooltip } from "./Tooltip";


interface MaybeTooltipProps {
    enabled: boolean;
    tooltip?: string;
    subTooltip?: string;
    children: React.ReactElement;
    content?: React.ReactNode;
    placement?: Placement;
    delay?: number;
}

export function MaybeTooltip({ enabled, tooltip, subTooltip, content, children, placement, delay }: MaybeTooltipProps) {
    if (!enabled) return children;
    return <Tooltip content={content} title={tooltip} subTitle={subTooltip} placement={placement} delay={delay}>{children}</Tooltip>;
}
