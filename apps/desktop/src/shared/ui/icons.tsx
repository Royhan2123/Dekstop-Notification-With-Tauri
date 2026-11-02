const base = {
  width: 20,
  height: 20,
  viewBox: "0 0 24 24",
  fill: "currentColor",
  "aria-hidden": true,
};

export const PrevIcon = () => (
  <svg {...base}>
    <path d="M6 5h2v14H6zM20 5v14L9 12z" />
  </svg>
);
export const NextIcon = () => (
  <svg {...base}>
    <path d="M16 5h2v14h-2zM4 5l11 7L4 19z" />
  </svg>
);
export const PlayIcon = () => (
  <svg {...base}>
    <path d="M7 4l13 8-13 8z" />
  </svg>
);
export const PauseIcon = () => (
  <svg {...base}>
    <path d="M6 4h4v16H6zM14 4h4v16h-4z" />
  </svg>
);
