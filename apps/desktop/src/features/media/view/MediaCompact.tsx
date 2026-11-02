import { useIslandStore } from "../../island/store";
import { useMediaVM } from "../viewmodel/useMediaVM";
import { Visualizer } from "./Visualizer";

export function MediaCompact() {
  const vm = useMediaVM(false);
  const content = useIslandStore((s) => s.snapshot.content);
  const { media } = vm;
  if (!media || !content) return null;
  return (
    <div className="island-body" onWheel={(e) => vm.onWheel(e.deltaY)}>
      {media.thumbnail && <img className="media-thumb-small" src={media.thumbnail} alt="" />}
      <span className="island-text">{content.text}</span>
      <Visualizer color={media.color} playing={media.playing} />
    </div>
  );
}
