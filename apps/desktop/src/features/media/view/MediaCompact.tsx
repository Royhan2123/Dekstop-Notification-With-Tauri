import { useIslandStore } from "../../island/store";
import { currentLyric, useLyricsVM } from "../viewmodel/useLyricsVM";
import { useMediaVM } from "../viewmodel/useMediaVM";
import { Visualizer } from "./Visualizer";

export function MediaCompact() {
  const lyrics = useLyricsVM();
  // Posisi hanya dihitung terus bila ada lirik, supaya pulau tanpa lirik tetap hemat CPU.
  const vm = useMediaVM(lyrics.length > 0);
  const content = useIslandStore((s) => s.snapshot.content);
  const { media } = vm;
  if (!media || !content) return null;
  return (
    <div className="island-body" onWheel={(e) => vm.onWheel(e.deltaY)}>
      {media.thumbnail && <img className="media-thumb-small" src={media.thumbnail} alt="" />}
      <span className="island-text">
        {(media.playing ? currentLyric(lyrics, vm.positionMs) : null) ?? content.text}
      </span>
      <Visualizer color={media.color} playing={media.playing} />
    </div>
  );
}
