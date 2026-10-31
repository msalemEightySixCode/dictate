// Dictate mark for the General tab. The component keeps its upstream name.
const HandyHand = ({
  width,
  height,
}: {
  width?: number | string;
  height?: number | string;
}) => (
  <svg
    width={width || 126}
    height={height || 135}
    viewBox="0 0 512 512"
    fill="none"
    strokeWidth="48"
    strokeLinecap="round"
    stroke="currentColor"
  >
    <g transform="rotate(90 256 256)">
      <path d="M163.9 189.8A92 92 0 0 1 163.9 322.2" />
      <path d="M218.1 133.7A170 170 0 0 1 218.1 378.3" />
      <path d="M272.3 77.6A248 248 0 0 1 272.3 434.4" />
    </g>
  </svg>
);

export default HandyHand;
