import type { SVGAttributes } from "preact";

export function IconXMark(props: SVGAttributes<SVGSVGElement>) {
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
      <path d="m4.5 4.5 7 7m0-7-7 7" stroke-width="1.5" />
    </svg>
  );
}
