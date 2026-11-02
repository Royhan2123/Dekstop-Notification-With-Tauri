import { commands, events, type IslandSnapshot, type MediaDto } from "../../bindings";

export { commands };
export type { IslandSnapshot, MediaDto };

/** Ambil snapshot awal lalu dengarkan perubahan. Mengembalikan fungsi untuk berhenti. */
export async function subscribeIsland(
  onSnapshot: (s: IslandSnapshot) => void,
): Promise<() => void> {
  const unlisten = await events.islandSnapshotEvent.listen((e) => onSnapshot(e.payload));
  onSnapshot(await commands.islandSnapshot());
  return unlisten;
}
