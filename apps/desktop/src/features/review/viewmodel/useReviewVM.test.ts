import { expect, it } from "vitest";
import type { FindingDto, ReviewDto } from "../../../shared/ipc";
import { shapeFor } from "../../island/viewmodel/useIslandVM";
import { idleSnapshot } from "../../island/store";
import { countBySeverity, sortedFindings } from "./useReviewVM";

const finding = (severity: FindingDto["severity"], title: string): FindingDto => ({
  file: "a.rs",
  line: 1,
  severity,
  title,
  detail: "",
  suggestion: null,
});

const report: ReviewDto = {
  summary: "s",
  patch_ready: false,
  findings: [
    finding("suggestion", "c"),
    finding("critical", "a"),
    finding("important", "b"),
    finding("critical", "d"),
  ],
};

it("menghitung temuan per tingkat", () => {
  expect(countBySeverity(report)).toEqual({ critical: 2, important: 1, suggestion: 1 });
});

it("mengurutkan dari yang terpenting tanpa mengubah laporan asli", () => {
  expect(sortedFindings(report).map((f) => f.title)).toEqual(["a", "d", "b", "c"]);
  expect(report.findings[0].title).toBe("c");
});

it("kartu review dengan temuan lebih besar daripada pulau biasa", () => {
  const plain = shapeFor({ ...idleSnapshot, mode: "expanded" });
  const withReview = shapeFor({ ...idleSnapshot, mode: "expanded", review: report });
  const clean = shapeFor({
    ...idleSnapshot,
    mode: "expanded",
    review: { ...report, findings: [] },
  });
  expect(withReview.height).toBeGreaterThan(plain.height);
  expect(clean).toEqual(plain);
});
