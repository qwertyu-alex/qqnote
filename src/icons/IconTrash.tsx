import type { SVGAttributes } from "preact";

export function IconTrash(props: SVGAttributes<SVGSVGElement>) {
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
      <path
        d="M2.75 4.25h10.5M6.25 4.25v-1.5h3.5v1.5M4.25 4.25l.55 8.3a1.5 1.5 0 0 0 1.5 1.4h3.4a1.5 1.5 0 0 0 1.5-1.4l.55-8.3M6.75 7v4.25M9.25 7v4.25"
        stroke-width="1.25"
      />
    </svg>
  );
}
