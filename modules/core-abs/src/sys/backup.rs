use std::fs;
use std::path::{Path, PathBuf};

pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    pub fn new() -> Self {
        // ~/.config/snt/backup
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
        path.push("snt");
        path.push("backup");
        Self { backup_dir: path }
    }

    pub fn backup(&self, category: &str, file_path_str: &str) -> Result<PathBuf, String> {
        let original_path = Path::new(file_path_str);
        if !original_path.exists() {
            return Err(format!("File {} does not exist.", file_path_str));
        }

        let mut category_dir = self.backup_dir.clone();
        category_dir.push(category);
        if !category_dir.exists() {
            fs::create_dir_all(&category_dir).map_err(|e| e.to_string())?;
        }

        let file_name = original_path.file_name().unwrap().to_string_lossy(); // generations
        let mut dest_path = category_dir.join(file_name.as_ref());
        let mut counter = 1;

        while dest_path.exists() {
            let new_name = format!("{}_{}", file_name, counter);
            dest_path = category_dir.join(new_name);
            counter += 1;
        }

        fs::copy(original_path, &dest_path).map_err(|e| e.to_string())?;
        Ok(dest_path)
    }

    pub fn clean(&self, category: Option<&str>) -> Result<(), String> {
        if let Some(cat) = category {
            let dir = self.backup_dir.join(cat);
            if dir.exists() {
                fs::remove_dir_all(dir).map_err(|e| e.to_string())?;
            }
        } else {
            if self.backup_dir.exists() {
                fs::remove_dir_all(&self.backup_dir).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}
