import type { SVGAttributes } from "preact";

export function IconRestore(props: SVGAttributes<SVGSVGElement>) {
  return (
    <svg
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      stroke-linecap="round"
      stroke-linejoin="round"
      height="1em"
      width="1em"
      {...props}
    >
      <path d="M3.5 6.5h6.25a3.25 3.25 0 0 1 0 6.5H7M3.5 6.5 6 4M3.5 6.5 6 9" stroke-width="1.3" />
    </svg>
  );
}
