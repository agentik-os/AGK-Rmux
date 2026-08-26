type Props = {
  title: string;
};

/** Thin running header for non-cover pages — small caps right-aligned, hairline below */
export function RunningHeader({ title }: Props) {
  return (
    <div className="running-header">
      <span>{title}</span>
    </div>
  );
}
