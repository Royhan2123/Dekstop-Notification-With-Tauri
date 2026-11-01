import { IslandShell } from "../../features/island/view/IslandShell";
import "./island.css";

export function IslandWindow() {
  return (
    <main className="island-stage">
      <IslandShell />
    </main>
  );
}
