interface ImageUploadProps {
  busy: boolean;
  onFile: (file: File) => void;
}

export function ImageUpload({ busy, onFile }: ImageUploadProps) {
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
          {busy ? "识别中" : "选择图片"}
          <input
            accept="image/*"
            disabled={busy}
            type="file"
            onChange={(event) => pickFile(event.currentTarget.files)}
          />
        </label>
      </div>
      <div
        className="drop-zone upload-drop-zone"
        onDragOver={(event) => event.preventDefault()}
        onDrop={(event) => {
          event.preventDefault();
          if (!busy) {
            pickFile(event.dataTransfer.files);
          }
        }}
      >
        <span>{busy ? "正在识别图片" : "拖入图片到这里"}</span>
      </div>
    </section>
  );
}
