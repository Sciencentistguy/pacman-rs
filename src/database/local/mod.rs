use std::{collections::HashMap, path::Path};

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

impl LocalDatabase {}

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

/// A lazy representation of the local database. It reads pacakges from the filesystem when they
/// are requested.
pub struct LocalDatabase {
    pub db: HashMap<String, LocalDatabaseEntry>,
    path: &'static Path, // Path::new("/var/lib/pacman/local")
}

impl LocalDatabase {
    pub fn new() -> Self {
        Self {
            db: HashMap::new(),
            path: Path::new("/var/lib/pacman/local"),
        }
    }

    pub fn pacakge_names(&self) -> impl Iterator<Item = &str> {
        self.db.iter().map(|(name, _)| name.as_str())
    }

    pub fn names(&self) -> Result<Vec<String>> {
        self.path
            .read_dir()?
            .filter_map(|x| -> Option<Result<_>> {
                match x {
                    Ok(x) => {
                        if is_valid_local_entry_dir(x.path()) {
                            Some(desc::read_desc_from_file(x.path().join("desc")).map(|x| x.name))
                        } else {
                            None
                        }
                    }
                    Err(e) => Some(Err(e.into())),
                }
            })
            .collect()
    }

    /// Retrieves a LocalDatabaseEntry for the pacakge with a specified name. If this package is
    /// present in the LazyLocalDatabase, it just returns a reference to it, otherwise it attempts
    /// to find the package in the pacman database. If it finds it, it reads it into the
    /// LazyLocalDatabase and then returns a reference to it. Returns Err(_) if the package cannot
    /// be found in the pacman database, or if there is an IO error
    pub fn get(&mut self, package_name: &str) -> Result<&LocalDatabaseEntry> {
        if let Some(entry) = self.db.get(package_name) {
            // safety: This is to work around a limitation of the compiler. These two pointers
            // never ailas so its fine to work around the borrow checker here
            Ok(unsafe { &*(entry as *const _) })
        } else {
            self.read_pacakge(package_name)
        }
    }

    /// Read the contents of a pacakge, by name
    pub fn read_pacakge(&mut self, package_name: &str) -> Result<&LocalDatabaseEntry> {
        for subdir in self.path.read_dir()? {
            let subdir = subdir?;
            if let Some(true) = subdir
                .file_name()
                .to_str()
                .map(|x| x.starts_with(package_name))
            {
                // Package found in filesystem
                let entry = LocalDatabaseEntry::new_from_directory(subdir.path())?;
                if entry.desc.name.as_str() != package_name {
                    continue;
                }
                self.db.insert(package_name.to_owned(), entry);

                return self.db.get(package_name).ok_or_else(|| unreachable!());
            }
        }
        Err("Could not find package '{}' in filesystem. Is it installed?".into())
    }

    pub fn populate(&mut self, query: &str) -> Result<()> {
        self.db.extend(self.path.read_dir()?.filter_map(|x| {
            if let Ok(x) = x {
                if is_valid_local_entry_dir(x.path()) {
                    if let Some(true) = x.file_name().to_str().map(|x| x.contains(query)) {
                        let x = match LocalDatabaseEntry::new_from_directory(x.path()) {
                            Ok(x) => x,
                            Err(_) => return None,
                        };
                        return Some((x.desc.name.clone(), x));
                    }
                }
            }
            None
        }));
        Ok(())
    }

    pub fn populate_full_database(&mut self) -> Result<()> {
        self.populate("")
    }
}

/// Reads the entire local database of a system, eagerly. This is rather slow.
//pub fn read_local_database() -> Result<LocalDatabase> {
//Path::new("/var/lib/pacman/local")
//.read_dir()?
//.filter_map(|path| {
//let path = match path {
//Ok(x) => x.path(),
//Err(e) => {
//eprintln!("Error reading directory in local database: '{}'", e);
//return None;
//}
//};
//if !is_valid_local_entry_dir(&path) {
//return None;
//}
//Some(LocalDatabaseEntry::new_from_directory(path))
//})
//.collect::<std::result::Result<Vec<_>, _>>()
//.map(|mut x| {
//x.sort_unstable_by(|left, right| left.desc.name.as_str().cmp(right.desc.name.as_str()));
//x
//})
//.map(LocalDatabase)
//}

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

    #[test]
    fn check_database_get() -> Result<()> {
        let mut lazy_db = LocalDatabase::new();
        let v = lazy_db.get("linux")?;
        println!("{:?}", v.desc);
        let x = lazy_db.get("vim")?;
        println!("{:?}", x.desc);
        Ok(())
    }

    #[test]
    fn check_database_query() -> Result<()> {
        let mut lazy_db = LocalDatabase::new();
        lazy_db.populate("linux")?;
        //assert_ne!(lazy_db.db, HashMap::new());
        for (name, _) in lazy_db.db.iter() {
            println!("{}", name);
        }
        Ok(())
    }
}
