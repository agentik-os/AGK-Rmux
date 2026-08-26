type Props = {
  left?: string;
  center?: string;
  right?: string;
  variant?: "cover" | "page";
};

export function FooterMeta({ left, center, right, variant = "page" }: Props) {
  const cls = variant === "cover" ? "caption" : "caption";
  return (
    <div className={`${cls} flex justify-between items-end no-break w-full`}>
      <span>{left || ""}</span>
      <span>{center || ""}</span>
      <span>{right || ""}</span>
    </div>
  );
}
