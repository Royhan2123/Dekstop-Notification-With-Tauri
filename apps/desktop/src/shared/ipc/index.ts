import {
  commands,
  events,
  type Decision,
  type FindingDto,
  type IslandSnapshot,
  type MediaDto,
  type ReviewDto,
  type Settings,
} from "../../bindings";

export { commands };
export type { Decision, FindingDto, IslandSnapshot, MediaDto, ReviewDto, Settings };

/** Ambil snapshot awal lalu dengarkan perubahan. Mengembalikan fungsi untuk berhenti. */
export async function subscribeIsland(
  onSnapshot: (s: IslandSnapshot) => void,
): Promise<() => void> {
  const unlisten = await events.islandSnapshotEvent.listen((e) => onSnapshot(e.payload));
  onSnapshot(await commands.islandSnapshot());
  return unlisten;
}

export async function subscribeSettings(onSettings: (s: Settings) => void): Promise<() => void> {
  const unlisten = await events.settingsChangedEvent.listen((e) => onSettings(e.payload));
  onSettings(await commands.settingsGet());
  return unlisten;
}
