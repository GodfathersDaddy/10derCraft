//! Bounded optional external scientific worker; no listener or shell execution.
use crate::{Result, params::bad};
use serde_json::Value;
#[cfg(target_arch = "wasm32")]
pub(super) fn execute(_: &Value) -> Result<Value> {
    Err(bad("mks.job.run", "local Exudyn execution is desktop only"))
}
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn execute(job: &Value) -> Result<Value> {
    use std::{
        fs::OpenOptions,
        io::{Read, Write},
        process::{Command, Stdio},
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    let worker = std::env::var_os("SOLIDCRAFT_ASSEMBLY_WORKER")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            let binary = std::env::current_exe().ok()?;
            binary.ancestors().map(|p| p.join("simulation/assembly_worker.py")).find(|p| p.is_file())
        })
        .ok_or_else(|| bad("mks.job.run", "optional simulation/assembly_worker.py missing"))?;
    let folder = worker.parent().ok_or_else(|| bad("mks.job.run", "invalid worker path"))?;
    let python = std::env::var_os("SOLIDCRAFT_SIMULATION_PYTHON")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| folder.join(if cfg!(windows) { ".venv/Scripts/python.exe" } else { ".venv/bin/python" }));
    if !python.is_file() {
        return Err(bad("mks.job.run", "optional scientific Python environment missing; see simulation/README.md"));
    }
    let payload = serde_json::to_vec(job).map_err(|e| bad("mks.job.run", e.to_string()))?;
    if payload.len() > 60 * 1024 {
        return Err(bad("mks.job.run", "job exceeds local prototype snapshot limit (60 KiB)"));
    }
    let mut command = Command::new(python);
    command.arg("-B").arg(&worker).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().map_err(|e| bad("mks.job.run", e.to_string()))?;
    let input = child.stdin.take();
    let output = child.stdout.take();
    let stderr = child.stderr.take();
    let bytes = payload.clone();
    let writer = std::thread::spawn(move || {
        if let Some(mut stream) = input {
            let _ = stream.write_all(&bytes);
        }
    });
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(stream) = output {
            stream.take(2_000_001).read_to_end(&mut bytes)?;
        }
        Ok::<_, std::io::Error>(bytes)
    });
    let errors = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(stream) = stderr {
            let _ = stream.take(65536).read_to_end(&mut bytes);
        }
        bytes
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if start.elapsed() < Duration::from_secs(30) => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
        }
    };
    let _ = writer.join();
    let bytes = reader.join().map_err(|_| bad("mks.job.run", "solver reader failed"))?.map_err(|e| bad("mks.job.run", e.to_string()))?;
    let stderr = errors.join().unwrap_or_default();
    // Archive each run, including failures, for reproducibility. Never overwrite a run.
    if let Some(root) = folder.parent() {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        let path = root.join("output/mks-runs").join(format!("{stamp}-{}", std::process::id()));
        if std::fs::create_dir_all(&path).is_ok() {
            for (name, data) in [("job.json", payload.as_slice()), ("result.json", bytes.as_slice()), ("stderr.txt", stderr.as_slice())] {
                if let Ok(mut f) = OpenOptions::new().write(true).create_new(true).open(path.join(name)) {
                    let _ = f.write_all(data);
                }
            }
        }
    }
    if status.is_none() {
        return Err(bad("mks.job.run", "local solver timed out after 30 s"));
    }
    if bytes.len() > 2_000_000 {
        return Err(bad("mks.job.run", "solver output exceeds 2 MB"));
    }
    let result: Value = serde_json::from_slice(&bytes).map_err(|e| bad("mks.job.run", format!("invalid solver result: {e}")))?;
    if !status.is_some_and(|s| s.success()) || result["status"] != "success" {
        return Err(bad("mks.job.run", result["error"].as_str().unwrap_or("solver failed; inspect output/mks-runs")));
    }
    Ok(result)
}
