import { useT } from "../../../shared/i18n";
import { useReviewVM } from "../viewmodel/useReviewVM";

/** Tombol keputusan: sebelum patch (Perbaiki/Lanjut/Batal) dan sesudah patch siap (Terapkan/Tolak). */
export function ReviewActions({ showDetail = false }: { showDetail?: boolean }) {
  const t = useT();
  const vm = useReviewVM();
  if (!vm.review) return null;

  // Klik tidak boleh sampai ke pulau (yang menangani klik untuk memperluas).
  const click = (fn: () => void) => (e: React.MouseEvent) => {
    e.stopPropagation();
    fn();
  };

  return (
    <div className="review-actions">
      {vm.review.patch_ready ? (
        <>
          <button
            type="button"
            className="review-primary"
            disabled={vm.busy}
            onClick={click(() => vm.decide("apply_patch"))}
          >
            {t("review.apply")}
          </button>
          <button type="button" disabled={vm.busy} onClick={click(() => vm.decide("reject_patch"))}>
            {t("review.reject")}
          </button>
        </>
      ) : (
        <>
          <button
            type="button"
            className="review-primary"
            disabled={vm.busy}
            onClick={click(() => vm.decide("fix"))}
          >
            {vm.busy ? t("review.working") : t("review.fix")}
          </button>
          <button type="button" disabled={vm.busy} onClick={click(() => vm.decide("continue"))}>
            {t("review.continue")}
          </button>
          <button type="button" disabled={vm.busy} onClick={click(() => vm.decide("cancel"))}>
            {t("review.cancel")}
          </button>
        </>
      )}
      {showDetail && (
        <button type="button" onClick={click(vm.openDetail)}>
          {t("review.detail")}
        </button>
      )}
    </div>
  );
}
