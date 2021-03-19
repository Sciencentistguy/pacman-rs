use std::path::{Path, PathBuf};

use crate::Result;

fn read_file<P: AsRef<Path>>(filepath: P) -> Result<Vec<PathBuf>> {
    let file = std::fs::read_to_string(filepath)?;
    let mut ret = Vec::new();
    for line in file.trim().split('\n').skip(1) {
        let path = PathBuf::from("/").join(line);
        println!("{:?}", path);
        println!("{:?}", path.is_file());
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
        let v = super::read_file("/var/lib/pacman/local/linux-5.11.6.arch1-1/files")?;
        println!("{:#?}", v);
        Ok(())
    }
}
