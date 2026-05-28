import { useEffect, useRef, useState } from "react";
import type { Detection, RecognitionResult } from "../../shared/types";
import { tileLabel } from "../../shared/tiles";

interface RecognitionPreviewProps {
  busy: boolean;
  imageUrl: string | null;
  recognition: RecognitionResult | null;
  selectedTileId: number | null;
  onTileSelect: (tileId: number | null) => void;
}

export function RecognitionPreview({
  busy,
  imageUrl,
  recognition,
  selectedTileId,
  onTileSelect,
}: RecognitionPreviewProps) {
  const detections = recognition?.detections ?? [];
  const imageRef = useRef<HTMLImageElement | null>(null);
  const [imageSize, setImageSize] = useState({
    naturalWidth: 1,
    naturalHeight: 1,
    displayWidth: 1,
    displayHeight: 1,
  });

  function updateImageSize() {
    const image = imageRef.current;
    if (!image) {
      return;
    }
    setImageSize({
      naturalWidth: image.naturalWidth || 1,
      naturalHeight: image.naturalHeight || 1,
      displayWidth: image.clientWidth || 1,
      displayHeight: image.clientHeight || 1,
    });
  }

  useEffect(() => {
    updateImageSize();
    const image = imageRef.current;
    if (!image) {
      return;
    }
    const observer = new ResizeObserver(updateImageSize);
    observer.observe(image);
    return () => observer.disconnect();
  }, [imageUrl]);

  const scaleX = imageSize.displayWidth / imageSize.naturalWidth;
  const scaleY = imageSize.displayHeight / imageSize.naturalHeight;

  return (
    <section className="panel preview-panel">
      <div className="panel-heading">
        <h2>识别标注</h2>
        <span className="meta">{detections.length} 张识别牌</span>
      </div>
      <div className="image-stage">
        {imageUrl ? (
          <img
            alt="识别标注预览"
            onLoad={updateImageSize}
            ref={imageRef}
            src={imageUrl}
          />
        ) : (
          <div />
        )}
        {detections.map((tile, index) => (
          <DetectionBox
            detection={tile}
            index={index}
            key={tile.id}
            role={recognition ? tileRole(recognition, tile.id) : "unassigned"}
            scaleX={scaleX}
            scaleY={scaleY}
            selected={selectedTileId === tile.id}
            onSelect={() => onTileSelect(tile.id)}
          />
        ))}
        {busy ? <div className="recognition-overlay">正在识别图片...</div> : null}
      </div>
    </section>
  );
}

function DetectionBox({
  detection,
  index,
  role,
  selected,
  scaleX,
  scaleY,
  onSelect,
}: {
  detection: Detection;
  index: number;
  role: TileRole;
  selected: boolean;
  scaleX: number;
  scaleY: number;
  onSelect: () => void;
}) {
  const labelOffset = LABEL_OFFSETS[index % LABEL_OFFSETS.length];

  return (
    <button
      className={`detection-box detection-${role}${selected ? " detection-selected" : ""}`}
      style={{
        left: `${detection.bbox.x * scaleX}px`,
        top: `${detection.bbox.y * scaleY}px`,
        width: `${detection.bbox.width * scaleX}px`,
        height: `${detection.bbox.height * scaleY}px`,
      }}
      title={`${tileLabel(detection.tile_id)} ${(detection.confidence * 100).toFixed(1)}%`}
      type="button"
      onClick={onSelect}
    >
      <span
        style={{
          transform: `translate(${labelOffset.x}px, ${labelOffset.y}px)`,
        }}
      >
        {tileLabel(detection.tile_id)}
      </span>
    </button>
  );
}

const LABEL_OFFSETS = [
  { x: 0, y: 0 },
  { x: 10, y: -10 },
  { x: -10, y: 10 },
  { x: 18, y: 8 },
  { x: -18, y: -8 },
  { x: 28, y: -18 },
];

type TileRole = "closed" | "winning" | "meld" | "unassigned";

function tileRole(recognition: RecognitionResult, tileId: number): TileRole {
  if (recognition.layout.hora === tileId) return "winning";
  if (recognition.layout.hand.includes(tileId)) return "closed";
  if (recognition.layout.naki.some((meld) => meld.tiles.includes(tileId))) return "meld";
  return "unassigned";
}
