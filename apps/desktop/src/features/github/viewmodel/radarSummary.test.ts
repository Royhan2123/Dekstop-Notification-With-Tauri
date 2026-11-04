import { describe, expect, it } from "vitest";
import type { RadarDto } from "../../../shared/ipc";
import { radarSummary } from "./radarSummary";

const pr = (ci: "passing" | "failing" | "pending" | "unknown") => ({
  repo: "a/b",
  title: "t",
  url: "https://github.com/a/b/pull/1",
  ci,
});

describe("radar GitHub", () => {
  it("meringkas permintaan review dan PR yang gagal CI", () => {
    const r: RadarDto = {
      review_requests: [pr("unknown"), pr("unknown")],
      mine: [pr("failing"), pr("passing"), pr("failing"), pr("pending")],
    };
    expect(radarSummary(r)).toEqual({ review: 2, failing: 2 });
    expect(radarSummary({ review_requests: [], mine: [] })).toEqual({ review: 0, failing: 0 });
  });
});
