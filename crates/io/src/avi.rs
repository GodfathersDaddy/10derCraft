//! Bounded single-stream Motion JPEG AVI 1.0 writer. Container layout follows
//! Microsoft's AVI RIFF file reference; no platform codec or external executable.
//! https://learn.microsoft.com/en-us/windows/win32/directshow/avi-riff-file-reference
const MAX_BYTES: usize = 128 * 1024 * 1024;
fn word(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn chunk(out: &mut Vec<u8>, id: &[u8; 4], bytes: &[u8]) {
    out.extend_from_slice(id);
    word(out, bytes.len() as u32);
    out.extend_from_slice(bytes);
    if bytes.len() % 2 == 1 {
        out.push(0);
    }
}
/// Encoded JPEG frames, indexed for seeking. Dimensions and allocation are capped.
pub fn mjpeg(width: u32, height: u32, fps: u32, frames: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    if !(64..=1024).contains(&width) || !(64..=1024).contains(&height) || !(1..=60).contains(&fps) || !(2..=601).contains(&frames.len()) {
        return Err("AVI requires 64..1024 pixels, 1..60 fps and 2..601 frames".into());
    }
    let bytes = frames
        .iter()
        .try_fold(0usize, |total, frame| {
            if !frame.starts_with(&[0xff, 0xd8]) || !frame.ends_with(&[0xff, 0xd9]) {
                return None;
            }
            total.checked_add(frame.len() + 32).filter(|v| *v <= MAX_BYTES - 4096)
        })
        .ok_or("Invalid JPEG frame or AVI exceeds 128 MiB")?;
    let max_frame = frames.iter().map(Vec::len).max().unwrap_or(0) as u32;
    let mut main = Vec::new();
    for n in [1_000_000 / fps, max_frame.saturating_mul(fps), 0, 0x10, frames.len() as u32, 0, 1, max_frame, width, height, 0, 0, 0, 0] {
        word(&mut main, n);
    }
    let mut stream = b"vidsMJPG".to_vec();
    for n in [0, 0, 0, 1, fps, 0, frames.len() as u32, max_frame, u32::MAX, 0] {
        word(&mut stream, n);
    }
    for n in [0i16, 0, width as i16, height as i16] {
        stream.extend_from_slice(&n.to_le_bytes());
    }
    let mut format = Vec::new();
    for n in [40, width, height, 0x0018_0001, u32::from_le_bytes(*b"MJPG"), width * height * 3, 0, 0, 0, 0] {
        word(&mut format, n);
    }
    let mut streams = b"strl".to_vec();
    chunk(&mut streams, b"strh", &stream);
    chunk(&mut streams, b"strf", &format);
    let mut header = b"hdrl".to_vec();
    chunk(&mut header, b"avih", &main);
    chunk(&mut header, b"LIST", &streams);
    let mut out = Vec::with_capacity(bytes + 4096);
    out.extend_from_slice(b"RIFF");
    word(&mut out, 0);
    out.extend_from_slice(b"AVI ");
    chunk(&mut out, b"LIST", &header);
    out.extend_from_slice(b"LIST");
    let movie_size = out.len();
    word(&mut out, 0);
    let movie_start = out.len();
    out.extend_from_slice(b"movi");
    let mut index = Vec::with_capacity(frames.len() * 16);
    for frame in frames {
        index.extend_from_slice(b"00dc");
        word(&mut index, 0x10);
        word(&mut index, (out.len() - movie_start) as u32);
        word(&mut index, frame.len() as u32);
        chunk(&mut out, b"00dc", frame);
    }
    let movie_len = (out.len() - movie_start) as u32;
    out[movie_size..movie_size + 4].copy_from_slice(&movie_len.to_le_bytes());
    chunk(&mut out, b"idx1", &index);
    let len = (out.len() - 8) as u32;
    out[4..8].copy_from_slice(&len.to_le_bytes());
    Ok(out)
}
