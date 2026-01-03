use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const TW_BINARY: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/tailwindcss_bin"));

pub fn get_tailwind_exe() -> Result<PathBuf> {
    let mut exe_path = std::env::temp_dir().join("ferrorpress_tailwind");

    if cfg!(target_os = "windows") {
        exe_path.set_extension("exe");
    }

    if !exe_path.exists() {
        let mut file = fs::File::create(&exe_path)?;
        file.write_all(TW_BINARY)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&exe_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&exe_path, perms)?;
        }
    }

    Ok(exe_path)
}
