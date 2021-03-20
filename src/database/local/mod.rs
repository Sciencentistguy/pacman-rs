use std::path::Path;

use crate::database::local::{desc::PackageDescription, mtree::MTreeEntry};
use crate::Result;

pub mod desc;
pub mod files;
pub mod mtree;

/// Represents an entry in the pacman local database (found in `/var/lib/pacman/local`). This
/// contains information about a specific installed pacakge, and the files it owns.
#[derive(Debug)]
pub struct LocalDatabaseEntry {
    pub desc: PackageDescription,
    pub mtree: Vec<MTreeEntry>,
}

pub struct LocalDatabase(Vec<LocalDatabaseEntry>);

impl LocalDatabase {
    pub fn iter(&self) -> impl Iterator<Item = &LocalDatabaseEntry> {
        self.0.iter()
    }
    pub fn pacakge_names(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|x| x.desc.name.as_str())
    }
}

impl LocalDatabaseEntry {
    /// Reads an entry in the database from a directory on disk. These entries are usually found in
    /// `/var/lib/pacman/local/*`. The directory must contain the files `desc` and `mtree`. Pacman
    /// also uses a file called `files`, but the data in there is a also contained in `mtree`, so
    /// it is not required
    pub fn new_from_directory<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();
        assert!(dir.is_dir());
        let desc = desc::read_desc_from_file(dir.join("desc"))?;
        let mtree = mtree::read_mtree_from_file(dir.join("mtree"))?;

        Ok(Self { desc, mtree })
    }

    /// Returns an iterator over std::path::Path objects of every file owned by the package.
    pub fn files(&self) -> impl Iterator<Item = &Path> {
        self.mtree
            .iter()
            .map(|x| std::path::Path::new(x.filepath.as_str()))
    }

    /// Check if the package owns a given file.
    pub fn owns<P: AsRef<Path>>(&self, file: P) -> bool {
        self.files().any(|x| x == file.as_ref())
    }
}

fn is_valid_local_entry_dir<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    path.is_dir() && path.join("desc").is_file() && path.join("mtree").is_file()
}

pub fn read_local_database() -> Result<LocalDatabase> {
    Path::new("/var/lib/pacman/local")
        .read_dir()?
        .filter_map(|path| {
            let path = match path {
                Ok(x) => x.path(),
                Err(e) => {
                    eprintln!("Error reading directory in local database: '{}'", e);
                    return None;
                }
            };
            if !is_valid_local_entry_dir(&path) {
                return None;
            }
            Some(LocalDatabaseEntry::new_from_directory(path))
        })
        .collect::<std::result::Result<Vec<_>, _>>()
        .map(|mut x| {
            x.sort_unstable_by(|left, right| left.desc.name.as_str().cmp(right.desc.name.as_str()));
            x
        })
        .map(|x| LocalDatabase(x))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Result;

    #[test]
    fn check_read_local_database_entry() -> Result<()> {
        let entry =
            LocalDatabaseEntry::new_from_directory("/var/lib/pacman/local/linux-5.11.6.arch1-1")?;
        println!("{:?}", entry);

        Ok(())
    }

    //Implicitly tests LocalDatabaseEntry::files()
    #[test]
    fn check_owns() -> Result<()> {
        let entry =
            LocalDatabaseEntry::new_from_directory("/var/lib/pacman/local/linux-5.11.6.arch1-1")?;
        let owns = entry
            .owns("/usr/lib/modules/5.11.6-arch1-1/kernel/arch/x86/crypto/aegis128-aesni.ko.xz");
        assert!(owns);
        Ok(())
    }
}
