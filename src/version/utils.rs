use std::path::{Path, PathBuf};
use home::home_dir;
use std::fs;


pub fn get_minecraft_dir() -> PathBuf {
    let mut path = home_dir().expect("Could not get home dir");
    path.push(".minecraft");
    tracing::trace!("Minecraft directory: {}", path.display());
    path
}

pub fn get_library_path(relative_path: &str) -> PathBuf {
    let mut path = get_minecraft_dir();
    path.push("libraries");
    path.push(relative_path);
    path
}

pub fn init_workspace() -> std::io::Result<()> {
    let base = get_minecraft_dir();
    
    let folders = [
        base.clone(),
        base.join("versions"),
        base.join("libraries"),
        base.join("assets"),
        base.join("assets/indexes"),
        base.join("assets/objects"),
    ];

    for folder in folders.iter() {
        if !folder.exists() {
            tracing::info!("Creating workspace folder: {:?}", folder);
            fs::create_dir_all(folder)?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn is_path_safe(target: &Path) -> bool {
    let base = get_minecraft_dir();
    target.starts_with(base)
}
