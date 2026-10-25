pub mod crypto;
pub mod protocol;

use std::path::PathBuf;

pub fn project_root() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let mut dir = exe.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
    loop {
        if dir.join("Cargo.toml").exists()
            || dir.join("static").is_dir()
            || dir.join("data").is_dir()
        {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    PathBuf::from(".")
}

pub fn data_dir() -> PathBuf {
    let d = project_root().join("data");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn tracks_dir(school_id: &str) -> PathBuf {
    let d = data_dir().join("tracks").join(school_id);
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn faces_dir() -> PathBuf {
    let d = data_dir().join("faces");
    let _ = std::fs::create_dir_all(&d);
    d
}
