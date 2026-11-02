import { useEffect, useState } from "react";
import { commands, subscribeIsland, type Decision, type ReviewDto } from "../../../shared/ipc";
import { useIslandStore } from "../../island/store";

type Severity = "critical" | "important" | "suggestion";

export function countBySeverity(report: ReviewDto): Record<Severity, number> {
  const counts = { critical: 0, important: 0, suggestion: 0 };
  for (const f of report.findings) counts[f.severity] += 1;
  return counts;
}

const RANK: Record<Severity, number> = { critical: 0, important: 1, suggestion: 2 };

/** Temuan terpenting dulu; urutan asli dipertahankan untuk yang setara. */
export function sortedFindings(report: ReviewDto) {
  return [...report.findings].sort((a, b) => RANK[a.severity] - RANK[b.severity]);
}

/** Jendela rincian punya konteks JS sendiri, jadi ikut berlangganan snapshot pulau. */
export function useReviewSync() {
  const setSnapshot = useIslandStore((s) => s.setSnapshot);
  useEffect(() => {
    const stop = subscribeIsland(setSnapshot);
    return () => {
      void stop.then((unlisten) => unlisten());
    };
  }, [setSnapshot]);
}

export function useReviewVM() {
  const review = useIslandStore((s) => s.snapshot.review);
  const [busy, setBusy] = useState(false);

  // Setiap laporan baru (patch siap, gagal, ditolak) membebaskan tombol lagi.
  useEffect(() => setBusy(false), [review]);

  return {
    review,
    busy,
    counts: review ? countBySeverity(review) : null,
    findings: review ? sortedFindings(review) : [],
    decide: (decision: Decision) => {
      setBusy(true);
      void commands.reviewDecide(decision);
    },
    openDetail: () => void commands.openReview(),
  };
}
