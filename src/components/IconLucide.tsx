import * as Icons from "lucide-react";
import type { LucideProps } from "lucide-react";

export type IconName = keyof typeof Icons;

export interface IconLucideProps extends LucideProps {
    name?: IconName;
}

export function IconLucide({
    name,
    ...props
}: IconLucideProps) {
    if (!name) {
        return null;
    }

    const Icon = Icons[name] as React.FC<LucideProps>;

    return Icon ? <Icon {...props} /> : null;
}