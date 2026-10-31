import React from "react";

// Dictate logo. The component keeps its upstream name to keep merges simple.
const HandyTextLogo = ({
  width,
  height,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => (
  <svg
    width={width}
    height={height}
    className={className}
    viewBox="50 110 1170 290"
    role="img"
    aria-label="Dictate"
  >
    <g
      transform="rotate(90 256 256)"
      fill="none"
      strokeWidth="44"
      strokeLinecap="round"
      className="stroke-text"
    >
      <path d="M163.9 189.8A92 92 0 0 1 163.9 322.2" />
      <path d="M218.1 133.7A170 170 0 0 1 218.1 378.3" />
      <path d="M272.3 77.6A248 248 0 0 1 272.3 434.4" />
    </g>
    <text
      x="480"
      y="340"
      className="fill-text"
      fontFamily="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
      fontSize="230"
      fontWeight="700"
      letterSpacing="-6"
    >
      dictate
    </text>
  </svg>
);

export default HandyTextLogo;
