use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use ort::session::Session;
use tauri::{path::BaseDirectory, AppHandle, Manager};

pub(crate) type VisionModel = Arc<Mutex<Session>>;
static MODEL: OnceLock<VisionModel> = OnceLock::new();

pub fn cached_model(app: &AppHandle, input_size: u32) -> Result<VisionModel, String> {
  if let Some(model) = MODEL.get() {
    return Ok(Arc::clone(model));
  }

  let loaded = load_onnx_model(app, input_size)?;
  if MODEL.set(Arc::clone(&loaded)).is_err() {
    return MODEL
      .get()
      .map(Arc::clone)
      .ok_or_else(|| "模型缓存初始化失败".to_string());
  }
  Ok(loaded)
}

fn load_onnx_model(app: &AppHandle, input_size: u32) -> Result<VisionModel, String> {
  load_model_from_path(model_path(app), input_size)
}

pub(crate) fn load_model_from_path(model_path: PathBuf, _input_size: u32) -> Result<VisionModel, String> {
  let mut builder = Session::builder().map_err(|err| format!("创建 ONNX Runtime Session 失败：{err}"))?;
  let session = builder
    .commit_from_file(&model_path)
    .map_err(|err| format!("加载 ONNX 模型失败（{}）：{err}", model_path.display()))?;
  Ok(Arc::new(Mutex::new(session)))
}

fn model_path(app: &AppHandle) -> PathBuf {
  let resource_path = app.path().resolve("model/last.onnx", BaseDirectory::Resource);
  if let Ok(path) = resource_path {
    if path.exists() {
      return path;
    }
  }

  source_model_path()
}

pub(crate) fn source_model_path() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("..")
    .join("..")
    .join("ml")
    .join("model")
    .join("last.onnx")
}
