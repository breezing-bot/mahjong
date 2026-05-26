interface ImageUploadProps {
  imageUrl: string | null;
  busy: boolean;
  onFile: (file: File) => void;
}

export function ImageUpload({ imageUrl, busy, onFile }: ImageUploadProps) {
  function pickFile(fileList: FileList | null) {
    const file = fileList?.[0];
    if (file) {
      onFile(file);
    }
  }

  return (
    <section className="panel upload-panel">
      <div className="panel-heading">
        <h2>照片</h2>
        <label className="file-button">
          选择图片
          <input
            accept="image/*"
            type="file"
            onChange={(event) => pickFile(event.currentTarget.files)}
          />
        </label>
      </div>
      <div
        className="drop-zone"
        onDragOver={(event) => event.preventDefault()}
        onDrop={(event) => {
          event.preventDefault();
          pickFile(event.dataTransfer.files);
        }}
      >
        {imageUrl ? (
          <img alt="牌桌照片预览" src={imageUrl} />
        ) : (
          <span>{busy ? "识别中" : "拖入牌桌照片"}</span>
        )}
      </div>
    </section>
  );
}
