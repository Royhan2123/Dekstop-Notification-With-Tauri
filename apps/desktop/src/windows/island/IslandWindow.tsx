import { IslandShell } from "../../features/island/view/IslandShell";
import { useSettingsSync } from "../../features/settings/viewmodel/useSettingsSync";
import "../../shared/theme/tokens.css";
import "./island.css";

export function IslandWindow() {
  useSettingsSync();
  return (
    <main className="island-stage">
      <IslandShell />
    </main>
  );
}
