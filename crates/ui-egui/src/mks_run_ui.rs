//! Local dynamics computes on a scratch session. Only the registered, source-
//! checked commit command applies its result; the user's CAD stays responsive.
use crate::SolveApp;
use serde_json::Value;
#[cfg(not(target_arch = "wasm32"))]
use serde_json::json;
use std::sync::{Arc, Mutex};
pub struct Running {
    pub inputs: Value,
    done: Mutex<Option<Result<Value, String>>>,
}
pub fn start(app: &SolveApp, ctx: &egui::Context, inputs: Value, recompute: bool) -> Result<Arc<Running>, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut scratch = app.session.scratch();
        let running = Arc::new(Running { inputs: inputs.clone(), done: Mutex::new(None) });
        let worker = running.clone();
        let ctx = ctx.clone();
        let mut inputs = inputs;
        inputs["recompute"] = json!(recompute);
        std::thread::Builder::new()
            .name("mks-dynamics".into())
            .spawn(move || {
                let outcome = (|| {
                    scratch.execute("mks.job.run", &inputs).map_err(|e| e.to_string())?;
                    let data = scratch.execute("mks.job.state", &json!({})).map_err(|e| e.to_string())?["data"].clone();
                    let mut result = data["result"].clone();
                    result["job"] = data["job"].clone();
                    Ok(result)
                })();
                if let Ok(mut done) = worker.done.lock() {
                    *done = Some(outcome);
                }
                ctx.request_repaint();
            })
            .map_err(|e| e.to_string())?;
        Ok(running)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (app, ctx, inputs, recompute);
        Err("Local dynamics is available on the desktop".into())
    }
}
impl Running {
    pub fn take(&self) -> Option<Result<Value, String>> {
        self.done.lock().ok().and_then(|mut v| v.take())
    }
}
