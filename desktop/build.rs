use std::{
    env,
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

const RT_ICON: u16 = 3;
const RT_GROUP_ICON: u16 = 14;
const LANG_NEUTRAL: u16 = 0;

fn u16(w: &mut impl Write, value: u16) -> std::io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

fn u32(w: &mut impl Write, value: u32) -> std::io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

fn write_res_entry(out: &mut impl Write, rtype: u16, id: u16, data: &[u8]) -> std::io::Result<()> {
    let header_size: u32 = 32;
    u32(out, data.len() as u32)?;
    u32(out, header_size)?;
    u16(out, 0xFFFF)?;
    u16(out, rtype)?;
    u16(out, 0xFFFF)?;
    u16(out, id)?;
    u32(out, 0)?;
    u16(out, 0x0010)?;
    u16(out, LANG_NEUTRAL)?;
    u32(out, 0)?;
    u32(out, data.len() as u32)?;

    out.write_all(data)?;
    let padding = (4 - (data.len() % 4)) % 4;
    out.write_all(&[0u8; 3][..padding as usize])
}

fn write_group_entry(out: &mut impl Write, png_len: u32) -> std::io::Result<()> {
    u16(out, 0)?;
    u16(out, 1)?;
    u16(out, 1)?;
    out.write_all(&[0, 0, 0, 0])?;
    u16(out, 1)?;
    u16(out, 32)?;
    u32(out, png_len)?;
    u16(out, 1)
}

fn write_res(path: &Path, png: &[u8]) -> std::io::Result<()> {
    let mut out = BufWriter::new(fs::File::create(path)?);

    u32(&mut out, 0)?;
    u32(&mut out, 32)?;
    u32(&mut out, 0xFFFF_FFFF)?;
    u32(&mut out, 0)?;
    u32(&mut out, 0)?;
    u16(&mut out, 0)?;
    u16(&mut out, 0)?;
    u32(&mut out, 0)?;
    u32(&mut out, 0)?;

    write_res_entry(&mut out, RT_ICON, 1, png)?;

    let mut group = Vec::new();
    write_group_entry(&mut group, png.len() as u32)?;
    write_res_entry(&mut out, RT_GROUP_ICON, 1, &group)?;

    out.flush()
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let windows_msvc = env::var("CARGO_CFG_WINDOWS").is_ok()
        && env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() == "msvc";
    if !windows_msvc {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let icon_path = manifest_dir.join("assets/icon.png");
    println!("cargo:rerun-if-changed={}", icon_path.display());

    let png = match fs::read(&icon_path) {
        Ok(png) => png,
        Err(e) => {
            println!("cargo:warning=missing {}", icon_path.display());
            println!("cargo:warning={e}");
            return;
        }
    };

    let res_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("mtorrent.res");
    if let Err(e) = write_res(&res_path, &png) {
        println!("cargo:warning=failed to write {}: {e}", res_path.display());
        return;
    }

    println!("cargo:rustc-link-arg-bins={}", res_path.display());
}
