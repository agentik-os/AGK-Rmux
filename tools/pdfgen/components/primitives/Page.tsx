import { ReactNode } from "react";

type Props = {
  children: ReactNode;
  variant?: "default" | "cover" | "bleed" | "flow";
  className?: string;
};

export function Page({ children, variant = "default", className = "" }: Props) {
  const v =
    variant === "cover" ? "pdf-page pdf-page--cover" :
    variant === "bleed" ? "pdf-page pdf-page--bleed" :
    variant === "flow"  ? "pdf-flow" :
    "pdf-page";
  return <section className={`${v} ${className}`}>{children}</section>;
}
