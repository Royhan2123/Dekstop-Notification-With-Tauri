import { useT } from "../../../shared/i18n";
import { useReviewVM } from "../viewmodel/useReviewVM";
import { ReviewActions } from "./ReviewActions";

const SHOWN = 2;

/** Kartu review di pulau yang membesar: ringkasan, temuan teratas, dan tombol keputusan. */
export function ReviewCard() {
  const t = useT();
  const vm = useReviewVM();
  if (!vm.review || !vm.counts) return null;
  const { counts, findings, review } = vm;

  return (
    <div className="review-card">
      <div className="review-counts">
        <span className="sev-critical">{counts.critical}</span>
        <span className="sev-important">{counts.important}</span>
        <span className="sev-suggestion">{counts.suggestion}</span>
        <span className="review-counts-label">
          {t("review.counts", counts.critical, counts.important, counts.suggestion)}
        </span>
      </div>
      <p className="review-summary">
        {review.patch_ready ? t("review.patch_ready") : review.summary}
      </p>
      <ul className="review-list">
        {findings.slice(0, SHOWN).map((f, i) => (
          <li key={`${f.file}:${f.line}:${i}`} className={`sev-${f.severity}`}>
            <span className="review-where">
              {f.file}
              {f.line ? `:${f.line}` : ""}
            </span>{" "}
            {f.title}
          </li>
        ))}
      </ul>
      {findings.length > SHOWN && (
        <span className="review-more">{t("review.more", findings.length - SHOWN)}</span>
      )}
      <ReviewActions showDetail />
    </div>
  );
}
