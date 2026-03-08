use std::path::{Path, PathBuf};

pub struct LocalFileStore {
    base_path: PathBuf,
}

impl LocalFileStore {
    pub fn new(base_path: impl Into<PathBuf>) -> Result<Self, String> {
        let base_path = base_path.into();
        std::fs::create_dir_all(&base_path)
            .map_err(|e| format!("Failed to create storage directory: {}", e))?;
        Ok(LocalFileStore { base_path })
    }

    pub fn file_path(&self, user_id: &str, file_id: &str) -> PathBuf {
        self.base_path.join(user_id).join(file_id)
    }

    pub fn temp_path(&self, user_id: &str, temp_id: &str) -> PathBuf {
        self.base_path.join(user_id).join(format!("tmp_{}", temp_id))
    }

    pub fn ensure_user_dir(&self, user_id: &str) -> Result<(), String> {
        std::fs::create_dir_all(self.base_path.join(user_id))
            .map_err(|e| format!("Failed to create user directory: {}", e))
    }

    #[allow(dead_code)]
    pub fn delete_file(&self, path: &Path) -> std::io::Result<()> {
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}
