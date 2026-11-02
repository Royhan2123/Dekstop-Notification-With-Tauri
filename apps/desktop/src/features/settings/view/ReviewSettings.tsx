import { useState } from "react";
import { useT } from "../../../shared/i18n";
import type { ReviewTimeoutAction } from "../../../bindings";
import { useReviewSettingsVM } from "../../review/viewmodel/useReviewSettingsVM";
import { useSettingsVM } from "../viewmodel/useSettingsVM";

const TIMEOUT_ACTIONS: ReviewTimeoutAction[] = ["continue", "cancel"];

/** Repo yang diawasi (hook pre-push) dan perilaku saat review tidak dijawab. */
export function ReviewSettings() {
  const t = useT();
  const vm = useSettingsVM();
  const review = useReviewSettingsVM();
  const [path, setPath] = useState("");
  if (!vm.settings) return null;
  const { settings } = vm;

  return (
    <fieldset className="field">
      <legend>{t("settings.review")}</legend>
      <p className="muted">{t("settings.review.hint")}</p>
      {settings.watched_repos.length === 0 ? (
        <p className="muted">{t("settings.review.none")}</p>
      ) : (
        <ul className="repo-list">
          {settings.watched_repos.map((repo) => (
            <li key={repo} className="row">
              <code>{repo}</code>
              <button type="button" disabled={review.busy} onClick={() => void review.unwatch(repo)}>
                {t("settings.review.unwatch")}
              </button>
            </li>
          ))}
        </ul>
      )}
      <div className="row">
        <input
          value={path}
          placeholder={t("settings.review.repo_placeholder")}
          onChange={(e) => setPath(e.currentTarget.value)}
        />
        <button
          type="button"
          disabled={review.busy || !path.trim()}
          onClick={() =>
            void review.watch(path).then((ok) => {
              if (ok) setPath("");
            })
          }
        >
          {t("settings.review.watch")}
        </button>
      </div>
      {review.error && (
        <p role="alert" className="error">
          {t("settings.error", review.error)}
        </p>
      )}
      <label className="column">
        {t("settings.review.timeout")}
        <select
          value={settings.review_timeout_action}
          onChange={(e) =>
            void vm.update({ review_timeout_action: e.currentTarget.value as ReviewTimeoutAction })
          }
        >
          {TIMEOUT_ACTIONS.map((a) => (
            <option key={a} value={a}>
              {t(`settings.review.timeout.${a}`)}
            </option>
          ))}
        </select>
      </label>
    </fieldset>
  );
}
