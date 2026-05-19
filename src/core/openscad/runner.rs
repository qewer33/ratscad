use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug)]
pub enum BuildError {
    Spawn(std::io::Error),
    NonZeroExit { code: Option<i32>, stderr: String },
    Parse(String),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn(err) => write!(f, "failed to spawn openscad: {err}"),
            Self::NonZeroExit { code, stderr } => {
                let code = code.map_or_else(|| "?".to_string(), |c| c.to_string());
                write!(f, "openscad exited with code {code}: {stderr}")
            }
            Self::Parse(msg) => write!(f, "stl parse error: {msg}"),
        }
    }
}

impl std::error::Error for BuildError {}

pub fn run_openscad(binary: &Path, source: &str) -> Result<Vec<u8>, BuildError> {
    let mut child = Command::new(binary)
        .args(["-", "-o", "-", "--export-format", "binstl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(BuildError::Spawn)?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(source.as_bytes())
            .map_err(BuildError::Spawn)?;
    }

    let output = child.wait_with_output().map_err(BuildError::Spawn)?;
    if !output.status.success() {
        return Err(BuildError::NonZeroExit {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    stl_to_obj(&output.stdout)
}

fn stl_to_obj(stl: &[u8]) -> Result<Vec<u8>, BuildError> {
    const HEADER: usize = 80;
    const COUNT_BYTES: usize = 4;
    const TRI_BYTES: usize = 50;

    if stl.len() < HEADER + COUNT_BYTES {
        return Err(BuildError::Parse(format!(
            "expected at least {} bytes, got {}",
            HEADER + COUNT_BYTES,
            stl.len()
        )));
    }
    let body_start = HEADER + COUNT_BYTES;
    let declared = u32::from_le_bytes(stl[HEADER..body_start].try_into().unwrap()) as usize;
    let by_size = (stl.len() - body_start) / TRI_BYTES;
    // OpenSCAD zeroes the STL triangle-count field when writing to stdout,
    // so we derive the real count from the file size whenever the declared
    // value doesn't match.
    let count = if declared > 0 && body_start + declared * TRI_BYTES == stl.len() {
        declared
    } else {
        by_size
    };
    let expected = body_start + count * TRI_BYTES;
    if stl.len() < expected {
        return Err(BuildError::Parse(format!(
            "truncated: expected {expected} bytes for {count} triangles, got {}",
            stl.len()
        )));
    }

    let mut out = Vec::with_capacity(count * 120);
    let mut idx: u32 = 1;
    for tri in 0..count {
        let tri_base = body_start + tri * TRI_BYTES;
        // Swap from OpenSCAD's Z-up frame to Bevy's Y-up frame: (x, y, z) becomes (x, z, -y).
        let nx = f32::from_le_bytes(stl[tri_base..tri_base + 4].try_into().unwrap());
        let ny = f32::from_le_bytes(stl[tri_base + 4..tri_base + 8].try_into().unwrap());
        let nz = f32::from_le_bytes(stl[tri_base + 8..tri_base + 12].try_into().unwrap());
        let (nx, ny, nz) = (nx, nz, -ny);
        let vbase = tri_base + 12;
        for v in 0..3 {
            let off = vbase + v * 12;
            let x = f32::from_le_bytes(stl[off..off + 4].try_into().unwrap());
            let y = f32::from_le_bytes(stl[off + 4..off + 8].try_into().unwrap());
            let z = f32::from_le_bytes(stl[off + 8..off + 12].try_into().unwrap());
            let (x, y, z) = (x, z, -y);
            writeln!(out, "v {x} {y} {z}").unwrap();
        }
        writeln!(out, "vn {nx} {ny} {nz}").unwrap();
        let n = tri as u32 + 1;
        writeln!(
            out,
            "f {}//{n} {}//{n} {}//{n}",
            idx,
            idx + 1,
            idx + 2
        )
        .unwrap();
        idx += 3;
    }
    Ok(out)
}
