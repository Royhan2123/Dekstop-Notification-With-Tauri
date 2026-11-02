import { useT } from "../../../shared/i18n";
import { NextIcon, PauseIcon, PlayIcon, PrevIcon } from "../../../shared/ui/icons";
import { useState } from "react";
import { useIslandStore } from "../../island/store";
import { formatTime, useMediaVM } from "../viewmodel/useMediaVM";
import { Visualizer } from "./Visualizer";

export function MediaExpanded() {
  const t = useT();
  const vm = useMediaVM(true);
  const content = useIslandStore((s) => s.snapshot.content);
  const { media } = vm;
  const [rate, setRate] = useState(1);
  const [query, setQuery] = useState("");
  if (!media || !content) return null;
  const ratio = media.duration_ms > 0 ? vm.positionMs / media.duration_ms : 0;
  return (
    <div
      className={`media-expanded${vm.extensionConnected ? " with-extras" : ""}`}
      onWheel={(e) => vm.onWheel(e.deltaY)}>
      {media.thumbnail ? (
        <img className="media-thumb" src={media.thumbnail} alt="" />
      ) : (
        <div className="media-thumb" />
      )}
      <div className="media-info">
        <div className="media-heading">
          <span className="island-text">{content.text}</span>
          <Visualizer color={media.color} playing={media.playing} />
        </div>
        <span className="island-detail">
          {content.detail}
          {media.session_count > 1 && ` · ${media.session_index + 1}/${media.session_count}`}
        </span>
        {media.duration_ms > 0 && (
          <div className="media-progress">
            <span>{formatTime(vm.positionMs)}</span>
            <div className="media-bar">
              <div
                className="media-bar-fill"
                style={{ width: `${Math.min(1, ratio) * 100}%`, background: media.color ?? "#fff" }}
              />
            </div>
            <span>{formatTime(media.duration_ms)}</span>
          </div>
        )}
        <div className="media-controls">
          <button type="button" aria-label={t("media.previous")} onClick={vm.onPrevious}>
            <PrevIcon />
          </button>
          <button
            type="button"
            aria-label={media.playing ? t("media.pause") : t("media.play")}
            onClick={vm.onToggle}
          >
            {media.playing ? <PauseIcon /> : <PlayIcon />}
          </button>
          <button type="button" aria-label={t("media.next")} onClick={vm.onNext}>
            <NextIcon />
          </button>
        </div>
        {vm.extensionConnected && (
          <div className="media-extras">
            <label className="media-rate">
              {t("media.rate")} {rate.toFixed(2)}x
              <input
                type="range"
                min={0.25}
                max={2}
                step={0.05}
                value={rate}
                onChange={(e) => setRate(Number(e.currentTarget.value))}
                onPointerUp={() => vm.onRate(rate)}
                onKeyUp={() => vm.onRate(rate)}
              />
            </label>
            <form
              className="media-search"
              onSubmit={(e) => {
                e.preventDefault();
                if (!query.trim()) return;
                vm.onSearch(query);
                setQuery("");
              }}
            >
              <input
                value={query}
                placeholder={t("media.search_placeholder")}
                onChange={(e) => setQuery(e.currentTarget.value)}
              />
              <button type="submit">{t("media.search")}</button>
            </form>
          </div>
        )}
      </div>
    </div>
  );
}
