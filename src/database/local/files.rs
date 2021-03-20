use std::path::{Path, PathBuf};

use crate::Result;

/// Reads a `files` file on disk, and returns a Vec of PathBufs to the files owned by the package.
/// This only works on packages that have been installed. This may change depending on how this
/// ends up being used.
pub fn read_files_from_file<P: AsRef<Path>>(filepath: P) -> Result<Vec<PathBuf>> {
    let file = std::fs::read_to_string(filepath)?;
    read_files(file.as_str().trim())
}

fn read_files(files: &str) -> Result<Vec<PathBuf>> {
    let mut ret = Vec::new();
    for line in files.trim().split('\n').skip(1) {
        let path = PathBuf::from("/").join(line);
        if path.is_file() {
            ret.push(path)
        }
    }
    Ok(ret)
}

#[cfg(test)]
mod test {
    use crate::Result;

    #[test]
    fn test_read_files() -> Result<()> {
        let v = super::read_files_from_file("/var/lib/pacman/local/linux-5.11.6.arch1-1/files")?;
        println!("{:#?}", v);
        Ok(())
    }
}
