import { useT } from "../../../shared/i18n";
import { useReviewVM } from "../viewmodel/useReviewVM";
import { ReviewActions } from "./ReviewActions";

/** Rincian lengkap semua temuan; dibuka dari tombol Rincian pada kartu review. */
export function DiffPanel() {
  const t = useT();
  const vm = useReviewVM();
  if (!vm.review) return <p className="muted">{t("review.empty")}</p>;

  return (
    <div className="review-panel">
      <h1>{t("review.window_title")}</h1>
      <p>{vm.review.summary}</p>
      <ul className="review-findings">
        {vm.findings.map((f, i) => (
          <li key={`${f.file}:${f.line}:${i}`} className={`finding sev-${f.severity}`}>
            <header>
              <span className="badge">{t(`review.${f.severity}`)}</span>
              <code>
                {f.file}
                {f.line ? `:${f.line}` : ""}
              </code>
            </header>
            <strong>{f.title}</strong>
            <p>{f.detail}</p>
            {f.suggestion && <pre>{f.suggestion}</pre>}
          </li>
        ))}
      </ul>
      <ReviewActions />
    </div>
  );
}
