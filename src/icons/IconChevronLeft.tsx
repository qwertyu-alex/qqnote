import type { SVGAttributes } from "preact";

export function IconChevronLeft(props: SVGAttributes<SVGSVGElement>) {
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
      <path d="M10 3 5 8l5 5" stroke-width="1.7" />
    </svg>
  );
}
