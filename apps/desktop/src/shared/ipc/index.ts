import {
  commands,
  events,
  type ClipEntryDto,
  type DayUsageDto,
  type Decision,
  type FindingDto,
  type HealthSummaryDto,
  type IslandSnapshot,
  type LyricLineDto,
  type MediaDto,
  type Mood,
  type PomodoroStatusDto,
  type AiActiveDto,
  type AiProvider,
  type RadarDto,
  type ReviewDto,
  type Settings,
  type TaskDto,
} from "../../bindings";

export { commands };
export type {
  AiActiveDto,
  AiProvider,
  RadarDto,
  ClipEntryDto,
  DayUsageDto,
  Decision,
  FindingDto,
  HealthSummaryDto,
  IslandSnapshot,
  LyricLineDto,
  MediaDto,
  Mood,
  PomodoroStatusDto,
  ReviewDto,
  Settings,
  TaskDto,
};

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
