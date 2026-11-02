/** Gelombang suara mungil; warnanya mengikuti sampul, bergerak hanya saat diputar. */
export function Visualizer({ color, playing }: { color: string | null; playing: boolean }) {
  return (
    <span
      className={playing ? "visualizer playing" : "visualizer"}
      style={{ color: color ?? "#fff" }}
      aria-hidden
    >
      <i />
      <i />
      <i />
      <i />
    </span>
  );
}
