use std::fs;
use std::path::Path;

use crate::error::Error;

/// `dir`の下のファイルを，`work_dir`からの`/`区切りのパスで返す．`dir`がファイルなら，そのファイルだけを返す．
/// `.git`ディレクトリは含めない．
pub fn list_files(work_dir: &Path, dir: &Path) -> Result<Vec<String>, Error> {
    let mut files = Vec::new();
    collect_files(work_dir, dir, &mut files)?;
    Ok(files)
}

fn collect_files(work_dir: &Path, path: &Path, files: &mut Vec<String>) -> Result<(), Error> {
    if path.is_dir() {
        let entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
        for entry in entries {
            if entry.file_name() != ".git" {
                collect_files(work_dir, &entry.path(), files)?;
            }
        }
    } else if path.is_file()
        && let Some(file) = relative_path(work_dir, path)
    {
        files.push(file);
    }
    Ok(())
}

/// `path`を，`work_dir`からの`/`区切りのパスにする．`work_dir`の外のパスなら`None`を返す．
pub fn relative_path(work_dir: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(work_dir).ok()?;
    let mut parts = Vec::new();
    for component in relative.components() {
        match component.as_os_str().to_str() {
            Some("..") | None => return None,
            Some(".") => {}
            Some(part) => parts.push(part),
        }
    }
    Some(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn work_tree() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join(".git/objects")).unwrap();
        fs::create_dir_all(dir.path().join("src/bin")).unwrap();
        fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();
        fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(dir.path().join("src/bin/tool.rs"), "fn main() {}\n").unwrap();
        dir
    }

    #[test]
    fn lists_files_recursively_without_git_directory() {
        let dir = work_tree();
        let mut files = list_files(dir.path(), dir.path()).unwrap();
        files.sort();
        assert_eq!(files, ["hello.txt", "src/bin/tool.rs", "src/main.rs"]);
    }

    #[test]
    fn lists_only_files_under_given_directory() {
        let dir = work_tree();
        let mut files = list_files(dir.path(), &dir.path().join("src/bin")).unwrap();
        files.sort();
        assert_eq!(files, ["src/bin/tool.rs"]);
    }

    #[test]
    fn lists_single_file() {
        let dir = work_tree();
        let files = list_files(dir.path(), &dir.path().join("hello.txt")).unwrap();
        assert_eq!(files, ["hello.txt"]);
    }

    #[test]
    fn relative_path_ignores_current_directory() {
        let dir = work_tree();
        let path = dir.path().join("src/./main.rs");
        assert_eq!(relative_path(dir.path(), &path).unwrap(), "src/main.rs");
        assert_eq!(
            relative_path(dir.path(), &dir.path().join(".")).unwrap(),
            ""
        );
    }

    #[test]
    fn path_outside_work_dir_is_an_error() {
        let dir = work_tree();
        let path = dir.path().join("src/../../x");
        assert_eq!(relative_path(dir.path(), &path), None);
    }
}
