//! One read-only result scene for desktop preview and reproducible PNG/GIF exports.
use super::CommandSpec;
use crate::schema::{TypedContract, output_schema_for};
use crate::{Result, Session, params::bad, view::Palette};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use solidcraft_doc::{Mat, apply_point, apply_vector};
use solidcraft_geom::{Aabb3, Vec3};
use solidcraft_render::{Camera, Rgb, Scene, SceneLine, SceneMesh, StandardView};
use std::sync::Arc;
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct FemSnapshotParams {
    path: String,
    #[serde(default)]
    scale: Option<f64>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    view: Option<String>,
    #[serde(default)]
    camera: Option<Value>,
    #[serde(default)]
    overwrite: Option<bool>,
}
#[derive(Serialize, JsonSchema)]
struct FemSnapshotOutput {
    path: String,
    scale: f64,
    width: usize,
    height: usize,
    bytes: usize,
    cad_changed: bool,
    max_display_displacement_mm: f64,
    field: String,
    model: String,
    camera: Value,
}
const FEM_SNAPSHOT_CONTRACT: TypedContract<FemSnapshotParams> = TypedContract::new();
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct VideoParams {
    path: String,
    width: Option<u32>,
    height: Option<u32>,
    fps: Option<u32>,
    quality: Option<u8>,
    view: Option<String>,
    camera: Option<Value>,
    overwrite: Option<bool>,
}
#[derive(Serialize, JsonSchema)]
struct VideoOutput {
    path: String,
    frames: usize,
    fps: u32,
    width: usize,
    height: usize,
    bytes: usize,
    source_duration_s: f64,
    cad_changed: bool,
    codec: String,
}
const VIDEO: TypedContract<VideoParams> = TypedContract::new();
pub static COMMANDS:&[CommandSpec]=&[
    CommandSpec::new("robot.fem.snapshot","Export Robot Beam Deformation PNG",crate::checked!("robot.fem.snapshot", fem_snapshot)).icon("save").contract(&FEM_SNAPSHOT_CONTRACT).output(output_schema_for::<FemSnapshotOutput>).noundo().params("path: new .png path, scale?:1 in 0..10000, width?:800,height?:500,view?:iso|front|top,camera?:Camera,overwrite?:false; current locked-actuator beam FEM; non-beam CAD is reference only"),
    CommandSpec::new("mks.result.snapshot","Export Dynamics PNG",snapshot).icon("save").noundo().params("sample: index, path: new .png path, width?:800,height?:500,view?:iso|front|top|...,camera?:full Camera object,overwrite?:false; renders stored current result without changing CAD"),
    CommandSpec::new("mks.result.animation","Export Dynamics GIF",animation).icon("timeline_play").noundo().params("path: new .gif path,width?:480,height?:320,view?:iso|front|top|...,speed?:1,overwrite?:false; all stored samples with one fixed camera, GIF delay rounded to 0.01 s"),
    CommandSpec::new("mks.result.video","Export Dynamics AVI",crate::checked!("mks.result.video", video)).icon("timeline_play").contract(&VIDEO).output(output_schema_for::<VideoOutput>).noundo(),
];
fn error(s: impl Into<String>) -> crate::EngineError {
    bad("mks.result.export", s)
}
/// Same geometry and poses for the UI and headless export; palette is display-only.
pub fn assembly_result_scene(s: &mut Session, sample: usize, palette: Palette) -> Result<(Scene, Aabb3)> {
    let frame = super::mks_simulation::frame(s, &json!({"sample":sample}))?;
    let model = s.model.state();
    let mut out = Scene { background: Some(palette.background), ..Scene::default() };
    let mut bounds = Aabb3::EMPTY;
    let mut count = 0;
    for placed in frame["bodies"].as_array().into_iter().flatten() {
        let body = model
            .bodies
            .iter()
            .find(|b| Some(b.name.as_str()) == placed["source"].as_str() && Some(s.doc.model_body_component(b)) == placed["component"].as_u64())
            .ok_or_else(|| error("CAD source body missing"))?;
        let world: Mat = serde_json::from_value(placed["world"].clone()).map_err(|e| error(e.to_string()))?;
        let source = body.mesh();
        count += source.triangles.len();
        if count > 100_000 || source.positions.len() > 500_000 {
            return Err(error("result preview limited to 100000 triangles / 500000 body vertices"));
        }
        let mut mesh = (*source).clone();
        for p in &mut mesh.positions {
            *p = apply_point(&world, *p);
            bounds.add(*p);
        }
        for n in &mut mesh.normals {
            *n = apply_vector(&world, *n);
        }
        for edge in &mut mesh.edges {
            for p in edge {
                *p = apply_point(&world, *p);
            }
        }
        let color = s
            .doc
            .body_color(body)
            .map(|c| Rgb((c[0] * 255.).round() as u8, (c[1] * 255.).round() as u8, (c[2] * 255.).round() as u8))
            .unwrap_or(palette.body);
        for edge in &mesh.edges {
            out.lines.push(SceneLine { points: edge.clone(), color: palette.edge, width: 1., alpha: 1., on_top: false });
        }
        out.meshes.push(SceneMesh::solid(Arc::new(mesh), color));
    }
    if bounds.is_empty() {
        return Err(error("no drawable CAD result geometry"));
    }
    let length = (bounds.diagonal() * 0.04).max(5.);
    for port in frame["ports"].as_array().into_iter().flatten() {
        let origin: Vec3 = serde_json::from_value(port["origin_mm"].clone()).map_err(|e| error(e.to_string()))?;
        for (i, axis) in port["axes"].as_array().into_iter().flatten().enumerate() {
            let axis: Vec3 = serde_json::from_value(axis.clone()).map_err(|e| error(e.to_string()))?;
            out.lines.push(SceneLine {
                points: vec![origin, origin + axis * length],
                color: palette.axes.get(i).copied().unwrap_or(palette.edge),
                width: 2.,
                alpha: 1.,
                on_top: true,
            });
        }
    }
    out.radius = bounds.diagonal().max(1.);
    Ok((out, bounds))
}
fn options<'a>(p: &'a Value, extension: &str, defaults: (usize, usize)) -> Result<(&'a str, usize, usize, Camera)> {
    let path = p
        .get("path")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty() && s.len() < 4096 && !s.chars().any(char::is_control) && s.to_ascii_lowercase().ends_with(extension))
        .ok_or_else(|| error(format!("path must end in {extension}")))?;
    let overwrite = match p.get("overwrite") {
        None => false,
        Some(Value::Bool(v)) => *v,
        _ => return Err(error("overwrite must be boolean")),
    };
    solidcraft_io::vfs::check(path).map_err(|e| error(e.to_string()))?;
    if !overwrite && solidcraft_io::vfs::exists(path) {
        return Err(error("destination exists; choose another path or explicitly set overwrite"));
    }
    let side = |key, default| -> Result<usize> {
        match p.get(key) {
            None => Ok(default),
            Some(v) => v
                .as_u64()
                .filter(|v| (64..=1024).contains(v))
                .map(|v| v as usize)
                .ok_or_else(|| error("width/height must be integer pixels in 64..1024")),
        }
    };
    let view = p.get("view").and_then(Value::as_str).unwrap_or("iso");
    let mut camera = Camera::default();
    camera.set_view(StandardView::parse(view).ok_or_else(|| error("unknown standard view"))?);
    if let Some(given) = p.get("camera") {
        camera = serde_json::from_value(given.clone()).map_err(|e| error(format!("invalid camera: {e}")))?;
        if !camera.is_valid() || camera.distance > 1e15 || camera.pitch.abs() > std::f64::consts::FRAC_PI_2 || !(0.0..3.0).contains(&camera.fov) {
            return Err(error("camera must be finite, with valid pitch, distance and field of view"));
        }
    }
    Ok((path, side("width", defaults.0)?, side("height", defaults.1)?, camera))
}
fn snapshot(s: &mut Session, p: &Value) -> Result<Value> {
    let (path, width, height, mut camera) = options(p, ".png", (800, 500))?;
    let sample = p.get("sample").and_then(Value::as_u64).filter(|v| *v < 1001).ok_or_else(|| error("sample index required"))? as usize;
    let (scene, bounds) = assembly_result_scene(s, sample, Palette::DEFAULT)?;
    if p.get("camera").is_none() {
        camera.fit(&bounds);
    }
    let png = solidcraft_render::render_png(&scene, &camera, width, height).ok_or_else(|| error("PNG encoding failed"))?;
    solidcraft_io::vfs::write(path, &png).map_err(|e| error(e.to_string()))?;
    Ok(json!({"path":path,"sample":sample,"width":width,"height":height,"bytes":png.len(),"cad_changed":false,"renderer":"cpu","camera":camera}))
}
fn animation(s: &mut Session, p: &Value) -> Result<Value> {
    let (path, width, height, mut camera) = options(p, ".gif", (480, 320))?;
    let speed = match p.get("speed") {
        None => 1.,
        Some(v) => v.as_f64().filter(|v| v.is_finite() && (0.05..=10.).contains(v)).ok_or_else(|| error("speed must be .05..10"))?,
    };
    let state = s.execute("mks.job.state", &json!({}))?;
    if state["result_current"] != true {
        return Err(error("current dynamics result required"));
    }
    let times = state["data"]["result"]["times_s"].as_array().ok_or_else(|| error("times missing"))?;
    let samples = times.len();
    let count = samples.min(101).min(8_000_000 / (width * height).max(1));
    if !(2..=1001).contains(&samples) || count < 2 {
        return Err(error("animation limited to 101 frames / 8 million total pixels; reduce width/height"));
    }
    let duration = times.last().and_then(Value::as_f64).ok_or_else(|| error("invalid interval"))?
        - times.first().and_then(Value::as_f64).ok_or_else(|| error("invalid interval"))?;
    let interval = duration / (count - 1) as f64;
    let indices: Vec<_> = (0..count).map(|i| i * (samples - 1) / (count - 1)).collect();
    let delay = (interval / speed * 100.).round().clamp(1., 65535.) as u16;
    let mut bounds = Aabb3::EMPTY;
    let mut triangles = 0;
    for &i in &indices {
        let (scene, b) = assembly_result_scene(s, i, Palette::DEFAULT)?;
        bounds = bounds.union(&b);
        triangles += scene.meshes.iter().map(|m| m.mesh.triangles.len()).sum::<usize>();
        if triangles > 2_000_000 {
            return Err(error("animation exceeds 2 million total frame triangles"));
        }
    }
    if p.get("camera").is_none() {
        camera.fit(&bounds);
    }
    let mut frames = Vec::new();
    for &i in &indices {
        let (mut scene, _) = assembly_result_scene(s, i, Palette::DEFAULT)?;
        scene.radius = bounds.diagonal().max(1.);
        frames.push(solidcraft_render::render(&scene, &camera, width, height).rgba);
    }
    let gif = solidcraft_render::gif(width, height, &frames, delay).ok_or_else(|| error("GIF encoding failed"))?;
    solidcraft_io::vfs::write(path, &gif).map_err(|e| error(e.to_string()))?;
    Ok(
        json!({"path":path,"frames":count,"width":width,"height":height,"bytes":gif.len(),"delay_s":delay as f64*0.01,"stored_interval_s":interval,"cad_changed":false,"renderer":"cpu","loop":true,"timing":"centisecond quantization; includes last-frame hold"}),
    )
}

fn video(s: &mut Session, p: &Value) -> Result<Value> {
    let input: VideoParams = serde_json::from_value(p.clone()).map_err(|e| error(e.to_string()))?;
    let (path, width, height, mut camera) = options(p, ".avi", (640, 360))?;
    let fps = input.fps.unwrap_or(30);
    let quality = input.quality.unwrap_or(85);
    if !(1..=60).contains(&fps) || !(1..=100).contains(&quality) {
        return Err(error("fps must be 1..60 and JPEG quality 1..100"));
    }
    let replay = super::mks_replay::prepare(s)?;
    let start = replay.times()[0];
    let duration = replay.times().last().copied().ok_or_else(|| error("times missing"))? - start;
    if !duration.is_finite() || duration <= 0. || duration > 10. {
        return Err(error("video trajectory must have a finite duration in 0..10 seconds"));
    }
    let count = (duration * f64::from(fps)).ceil() as usize + 1;
    if !(2..=601).contains(&count) || width * height * count > 150_000_000 {
        return Err(error("video limited to 601 frames / 150 million pixels; reduce resolution or fps"));
    }
    if input.camera.is_none() {
        camera.fit(&replay.bounds);
    }
    let mut frames = Vec::with_capacity(count);
    let mut encoded_bytes = 0;
    for i in 0..count {
        let time = (start + i as f64 / f64::from(fps)).min(start + duration);
        let mut scene = Scene { background: Some(Palette::DEFAULT.background), radius: replay.bounds.diagonal().max(1.), ..Scene::default() };
        for (source, world) in replay.meshes.iter().zip(replay.worlds(time)) {
            let mut mesh = (*source.mesh).clone();
            for p in &mut mesh.positions {
                *p = apply_point(&world, *p);
            }
            for n in &mut mesh.normals {
                *n = apply_vector(&world, *n);
            }
            mesh.edges.clear();
            scene.meshes.push(SceneMesh::solid(Arc::new(mesh), source.color));
        }
        let pixels = solidcraft_render::render(&scene, &camera, width, height);
        let rgba = image::RgbaImage::from_raw(width as u32, height as u32, pixels.rgba).ok_or_else(|| error("invalid frame dimensions"))?;
        let rgb = image::DynamicImage::ImageRgba8(rgba).into_rgb8();
        let mut jpeg = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, quality).encode_image(&rgb).map_err(|e| error(e.to_string()))?;
        encoded_bytes += jpeg.len() + 32;
        if encoded_bytes > 127 * 1024 * 1024 {
            return Err(error("video exceeds 128 MiB"));
        }
        frames.push(jpeg);
    }
    let avi = solidcraft_io::avi::mjpeg(width as u32, height as u32, fps, &frames).map_err(error)?;
    solidcraft_io::vfs::write(path, &avi).map_err(|e| error(e.to_string()))?;
    Ok(json!(VideoOutput {
        path: path.into(),
        frames: count,
        fps,
        width,
        height,
        bytes: avi.len(),
        source_duration_s: duration,
        cad_changed: false,
        codec: "MJPEG / AVI".into()
    }))
}

fn fem_snapshot(s: &mut Session, p: &Value) -> Result<Value> {
    let (path, width, height, mut camera) = options(p, ".png", (800, 500))?;
    let scale = match p.get("scale") {
        None => 1.,
        Some(v) => v.as_f64().ok_or_else(|| error("scale must be a number"))?,
    };
    let (scene, bounds, maximum) = super::robot_fem::result_scene(s, scale, Palette::DEFAULT)?;
    if p.get("camera").is_none() {
        camera.fit(&bounds);
    }
    let png = solidcraft_render::render_png(&scene, &camera, width, height).ok_or_else(|| error("PNG encoding failed"))?;
    solidcraft_io::vfs::write(path, &png).map_err(|e| error(e.to_string()))?;
    Ok(
        json!({"path":path,"scale":scale,"width":width,"height":height,"bytes":png.len(),"cad_changed":false,"max_display_displacement_mm":maximum,"field":"displacement magnitude, blue-green-red; display amplification does not change scalar values","model":"locked-actuator beam FEM; non-beam CAD is undeformed reference","camera":camera}),
    )
}
