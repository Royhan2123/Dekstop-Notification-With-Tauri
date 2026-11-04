import type { RadarDto } from "../../../shared/ipc";

/** Ringkasan satu baris: berapa PR menunggu review kita dan berapa PR kita yang gagal CI. */
export function radarSummary(r: RadarDto): { review: number; failing: number } {
  return {
    review: r.review_requests.length,
    failing: r.mine.filter((p) => p.ci === "failing").length,
  };
}
