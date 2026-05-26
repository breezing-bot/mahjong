use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tauri::{path::BaseDirectory, AppHandle, Manager};
use tract_onnx::prelude::*;

type VisionModel = Arc<TypedRunnableModel<TypedModel>>;
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
    let model_path = model_path(app);
    let model = tract_onnx::onnx()
        .model_for_path(model_path)
        .map_err(|err| format!("加载 ONNX 模型失败：{err}"))?
        .with_input_fact(
            0,
            f32::fact([1, 3, input_size as usize, input_size as usize]).into(),
        )
        .map_err(|err| err.to_string())?
        .into_optimized()
        .map_err(|err| err.to_string())?
        .into_runnable()
        .map_err(|err| err.to_string())?;
    Ok(Arc::new(model))
}

fn model_path(app: &AppHandle) -> PathBuf {
    let resource_path = app
        .path()
        .resolve("model/last.onnx", BaseDirectory::Resource);
    if let Ok(path) = resource_path {
        if path.exists() {
            return path;
        }
    }

    source_model_path()
}

fn source_model_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("ml")
        .join("model")
        .join("last.onnx")
}
