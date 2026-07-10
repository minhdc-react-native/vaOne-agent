import * as Icons from 'lucide-react';
import type { LucideProps } from "lucide-react";
export type IconName = keyof typeof Icons;

export function IconLucide({ name, ...props }: { name: IconName;[key: string]: any }) {
    const Icon = Icons[name] as React.FC<LucideProps>;
    return Icon ? <Icon {...props} /> : null;
}
