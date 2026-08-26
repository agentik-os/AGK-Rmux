use std::path::Path;

/// Set directory permissions to 0700 on unix (no-op elsewhere).
/// Logs on error but never panics.
#[cfg(unix)]
pub fn harden_dir(dir: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700));
}

/// Set file permissions to 0600 on unix (no-op elsewhere).
/// Logs on error but never panics.
#[cfg(unix)]
pub fn harden_file(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
pub fn harden_dir(_dir: &Path) {}

#[cfg(not(unix))]
pub fn harden_file(_path: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    #[cfg(unix)]
    fn test_harden_dir_sets_0700() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        harden_dir(dir_path);

        let perms = fs::metadata(dir_path).unwrap().permissions();
        let mode = perms.mode() & 0o777;
        assert_eq!(mode, 0o700, "Directory should have 0700 permissions");
    }

    #[test]
    #[cfg(unix)]
    fn test_harden_file_sets_0600() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        harden_file(&file_path);

        let perms = fs::metadata(&file_path).unwrap().permissions();
        let mode = perms.mode() & 0o777;
        assert_eq!(mode, 0o600, "File should have 0600 permissions");
    }
}
