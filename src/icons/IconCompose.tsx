import type { SVGAttributes } from "preact";

export function IconCompose(props: SVGAttributes<SVGSVGElement>) {
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
        d="M8 2.75H4.25a1.5 1.5 0 0 0-1.5 1.5v7.5a1.5 1.5 0 0 0 1.5 1.5h7.5a1.5 1.5 0 0 0 1.5-1.5V8M11.6 2.4a1.27 1.27 0 0 1 1.8 1.8L8.2 9.4l-2.4.6.6-2.4z"
        stroke-width="1.25"
      />
    </svg>
  );
}
