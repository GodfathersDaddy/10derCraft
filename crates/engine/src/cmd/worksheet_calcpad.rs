//! Local one-shot Calcpad process; never opens a service or a network listener.
use super::worksheet_references::CALCPAD;
use crate::params::{bad, str_};
use crate::{Result, Session};
use serde_json::{Value, json};

pub fn edit(s: &mut Session, p: &Value) -> Result<Value> {
    let source = str_(p, "source").ok_or_else(|| bad("worksheet.calcpad.edit", "source required"))?;
    if source.len() > 128 * 1024 {
        return Err(bad("worksheet.calcpad.edit", "source exceeds 128 KiB"));
    }
    let mut state = s.doc.metadata.get(CALCPAD).cloned().unwrap_or_else(|| json!({}));
    state["source"] = json!(source);
    state["dirty"] = json!(true);
    s.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, CALCPAD, state.clone())?;
    Ok(state)
}

pub fn calculate(s: &mut Session, p: &Value) -> Result<Value> {
    let source = str_(p, "source").or_else(|| s.doc.metadata.get(CALCPAD).and_then(|m| m["source"].as_str())).unwrap_or("").to_owned();
    if source.len() > 128 * 1024 {
        return Err(bad("worksheet.calcpad.calculate", "source exceeds 128 KiB"));
    }
    let result = local(&source, &super::worksheet_inline::requests(s))?;
    if result["ok"] != true {
        return Err(bad("worksheet.calcpad.calculate", result["error"].as_str().unwrap_or("local calculation failed")));
    }
    let state = json!({"source":source,"evaluated_source":source,"dirty":false,"backend":"CalcpadCE",
        "results":result["results"],"html":result["html"]});
    s.doc_mut().metadata.set(solidcraft_doc::metadata::Owner::Design, CALCPAD, state.clone())?;
    super::worksheet_inline::display_results(s, &result)?;
    Ok(state)
}

#[cfg(target_arch = "wasm32")]
pub(super) fn local(_: &str, _: &Value) -> Result<Value> {
    Err(bad("worksheet.calcpad.calculate", "local Calcpad is available in the desktop application"))
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn local(source: &str, units: &Value) -> Result<Value> {
    use std::{
        io::{Read, Write},
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    let executable = std::env::var_os("SOLIDCRAFT_CALCPAD_HELPER")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            let binary = std::env::current_exe().ok()?;
            binary.ancestors().map(|p| p.join("output/calcpad-local/CalcpadLocal.exe")).find(|p| p.is_file())
        })
        .ok_or_else(|| bad("worksheet.calcpad.calculate", "local Calcpad helper missing; run Build-Calcpad-Local.ps1"))?;
    let mut command = Command::new(executable);
    command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().map_err(|e| bad("worksheet.calcpad.calculate", e.to_string()))?;
    let payload = serde_json::to_vec(&json!({"source":source,"units":units})).map_err(|e| bad("worksheet.calcpad.calculate", e.to_string()))?;
    let input = child.stdin.take();
    let output = child.stdout.take();
    let writer = std::thread::spawn(move || {
        if let Some(mut input) = input {
            let _ = input.write_all(&payload);
        }
    });
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(output) = output {
            output.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
        }
        Ok::<_, std::io::Error>(bytes)
    });
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() < Duration::from_secs(10) => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = writer.join();
                let _ = reader.join();
                return Err(bad("worksheet.calcpad.calculate", "local calculation timed out or process failed"));
            }
        }
    }
    let _ = writer.join();
    let bytes = reader
        .join()
        .map_err(|_| bad("worksheet.calcpad.calculate", "result reader failed"))?
        .map_err(|e| bad("worksheet.calcpad.calculate", e.to_string()))?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(bad("worksheet.calcpad.calculate", "result exceeds 4 MiB"));
    }
    serde_json::from_slice(&bytes).map_err(|e| bad("worksheet.calcpad.calculate", format!("invalid local result: {e}")))
}
