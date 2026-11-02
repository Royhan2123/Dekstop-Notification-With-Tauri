import { DiffPanel } from "../../features/review/view/DiffPanel";
import { useReviewSync } from "../../features/review/viewmodel/useReviewVM";
import { useSettingsSync } from "../../features/settings/viewmodel/useSettingsSync";
import "./review.css";

export function ReviewWindow() {
  useSettingsSync();
  useReviewSync();
  return <DiffPanel />;
}
