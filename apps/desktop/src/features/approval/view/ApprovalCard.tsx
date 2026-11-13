import { useState } from "react";
import { useT } from "../../../shared/i18n";
import { commands } from "../../../shared/ipc";

interface Props {
  /** Id permintaan dari Jarvis; dipakai menjawab lewat API lokal Jarvis. */
  id: string;
  text: string;
}

/** Izin dari Jarvis (agen ingin menjalankan sesuatu): Setuju atau Tolak; tak dijawab = ditolak. */
export function ApprovalCard({ id, text }: Props) {
  const t = useT();
  const [error, setError] = useState(false);
  const [busy, setBusy] = useState(false);

  const decide = async (allow: boolean) => {
    setBusy(true);
    setError(false);
    const res = await commands.approvalDecide(id, allow);
    // Sukses menutup kartu dari backend; galat (Jarvis mati / kartu kedaluwarsa) ditampilkan di sini.
    if (res.status === "error") {
      setError(true);
      setBusy(false);
    }
  };

  return (
    <div className="island-body expanded approval-card" role="alertdialog" aria-label={text}>
      <span className="island-label">{t("island.approval")}</span>
      <span className="island-text">{text}</span>
      {error && <span className="approval-error">{t("approval.failed")}</span>}
      <div className="approval-actions">
        <button
          type="button"
          className="island-dismiss approval-deny"
          disabled={busy}
          onClick={(e) => {
            e.stopPropagation();
            void decide(false);
          }}
        >
          {t("approval.deny")}
        </button>
        <button
          type="button"
          className="island-dismiss approval-allow"
          disabled={busy}
          onClick={(e) => {
            e.stopPropagation();
            void decide(true);
          }}
        >
          {t("approval.allow")}
        </button>
      </div>
    </div>
  );
}
