import { useEffect, useRef, useState } from "react";
import type { Detection, RecognitionResult } from "../../shared/types";
import { tileLabel } from "../../shared/tiles";

interface RecognitionPreviewProps {
  imageUrl: string | null;
  recognition: RecognitionResult | null;
}

export function RecognitionPreview({
  imageUrl,
  recognition,
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
        <h2>识别</h2>
        <span className="meta">{detections.length} 张</span>
      </div>
      <div className="image-stage">
        {imageUrl ? (
          <img
            alt="检测框预览"
            onLoad={updateImageSize}
            ref={imageRef}
            src={imageUrl}
          />
        ) : (
          <div />
        )}
        {detections.map((detection, index) => (
          <DetectionBox
            detection={detection}
            key={`${detection.tile_id}-${index}`}
            scaleX={scaleX}
            scaleY={scaleY}
          />
        ))}
      </div>
      {recognition ? (
        <div className="quality-row">
          {recognition.quality_flags.length === 0
            ? "识别质量正常"
            : recognition.quality_flags.join(" / ")}
        </div>
      ) : null}
    </section>
  );
}

function DetectionBox({
  detection,
  scaleX,
  scaleY,
}: {
  detection: Detection;
  scaleX: number;
  scaleY: number;
}) {
  return (
    <div
      className="detection-box"
      style={{
        left: `${detection.bbox.x * scaleX}px`,
        top: `${detection.bbox.y * scaleY}px`,
        width: `${detection.bbox.width * scaleX}px`,
        height: `${detection.bbox.height * scaleY}px`,
      }}
      title={`${tileLabel(detection.tile_id)} ${(detection.confidence * 100).toFixed(1)}%`}
    >
      <span>{tileLabel(detection.tile_id)}</span>
    </div>
  );
}
