import { useIslandStore } from "../../island/store";
import { currentLyric, useLyricsVM } from "../viewmodel/useLyricsVM";
import { LYRIC_TICK_MS, useMediaVM } from "../viewmodel/useMediaVM";
import { KaraokeLine } from "./KaraokeLine";
import { Visualizer } from "./Visualizer";

export function MediaCompact() {
  const lyrics = useLyricsVM();
  // Posisi hanya dihitung terus bila ada lirik, supaya pulau tanpa lirik tetap hemat CPU.
  const vm = useMediaVM(lyrics.length > 0, LYRIC_TICK_MS);
  const content = useIslandStore((s) => s.snapshot.content);
  const { media } = vm;
  if (!media || !content) return null;
  const lyric = media.playing ? currentLyric(lyrics, vm.positionMs) : null;
  return (
    <div className="island-body" onWheel={(e) => vm.onWheel(e.deltaY)}>
      {media.thumbnail && <img className="media-thumb-small" src={media.thumbnail} alt="" />}
      {lyric ? (
        <KaraokeLine text={lyric.text} fraction={lyric.fraction} />
      ) : (
        <span className="island-text">{content.text}</span>
      )}
      <Visualizer color={media.color} playing={media.playing} />
    </div>
  );
}
