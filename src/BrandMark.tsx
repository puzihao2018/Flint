import mark from "../branding/mark.json";

export function BrandMark({
  size = 24,
  monochrome = false,
}: {
  size?: number;
  monochrome?: boolean;
}) {
  return (
    <svg
      width={size}
      height={size}
      viewBox={mark.viewBox.join(" ")}
      aria-hidden="true"
      style={{ flexShrink: 0 }}
    >
      {mark.bars.map((bar, index) => (
        <rect
          key={index}
          {...bar}
          fill={monochrome ? "currentColor" : bar.fill}
        />
      ))}
    </svg>
  );
}
