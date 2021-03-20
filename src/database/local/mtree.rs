use std::io::prelude::*;
use std::path::Path;

use crate::Result;

/// Represents a single entry in an `mtree` file. This contains information about a single file
/// owned by a single pacakge.
#[derive(Debug)]
pub struct MTreeEntry {
    /// The path of the file. Pacman seems to use relative paths from root, but it is much easier
    /// to work with absolute paths instead, so the leading `.` is stripped
    pub filepath: String,
    /// The checksum of the file.
    pub hashes: Hashes,
    /// The unix permissions of the file.
    pub mode: u16,
    /// The group that owns the file.
    pub gid: u32,
    /// The user that owns the file.
    pub uid: u32,
    /// The time that the file was created.
    pub time: u64,
    /// The size of the file, in bytes.
    pub filesize: usize,
    /// The type of the file.
    pub filetype: FileType,
    /// If the type is SymbolicLink, the target of the link, relative from the position of the file
    /// itself.
    pub link: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    Directory,
    File,
    SymbolicLink,

    None,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Hashes {
    md5: Option<String>,
    sha256: Option<String>,
}

/// Reads an `mtree` file from disk, and returns a Vec of the parsed data.
pub fn read_mtree_from_file<P: AsRef<Path>>(filepath: P) -> Result<Vec<MTreeEntry>> {
    let mtree = {
        let gzipped_bytes = std::fs::read(filepath)?;
        let mut decoder = flate2::read::GzDecoder::new(&*gzipped_bytes);
        let mut s = String::new();
        decoder.read_to_string(&mut s)?;
        s
    };
    read_mtree(mtree.as_str())
}

fn read_mtree(mtree: &str) -> Result<Vec<MTreeEntry>> {
    let mut ret = Vec::new();
    let mut mode = 0o0000;
    let mut gid = 0;
    let mut uid = 0;
    let mut filesize = 0;
    for line in mtree.trim().split('\n') {
        let mut filepath = None;
        let mut hashes = Hashes {
            md5: None,
            sha256: None,
        };
        let mut link = None;
        let mut time = 0;
        let mut filetype = FileType::None;

        for section in line.trim().split(' ').map(|x| x.trim()) {
            if !section.contains('=') {
                if section.starts_with("/set") || section == "#mtree" {
                    continue;
                } else {
                    filepath = if section.starts_with(".") {
                        section.strip_prefix('.').map(|x| x.to_owned())
                    } else {
                        Some(section.to_owned())
                    };
                    continue;
                }
            }
            let (first, second) = {
                let mut it = section.split('=');
                (it.next().unwrap().trim(), it.next().unwrap().trim())
            };
            match first {
                "mode" => mode = second.parse()?,
                "gid" => gid = second.parse()?,
                "uid" => uid = second.parse()?,
                "size" => filesize = second.parse()?,
                "time" => time = second.parse::<f64>()? as u64,
                "link" => link = Some(second.to_owned()),
                "type" => {
                    filetype = match second {
                        "file" => FileType::File,
                        "dir" => FileType::Directory,
                        "link" => FileType::SymbolicLink,
                        _ => {
                            return Err(format!(
                                "Unknown filetype '{}' found in path '{}'",
                                second,
                                filepath.unwrap()
                            )
                            .into())
                        }
                    }
                }

                x if first.contains("digest") => {
                    let x = x.strip_suffix("digest").unwrap();
                    match x {
                        "md5" => hashes.md5 = Some(second.to_owned()),
                        "sha256" => hashes.sha256 = Some(second.to_owned()),
                        _ => return Err(format!("Unknown hash type '{}' specified", second).into()),
                    }
                }

                x => {
                    return Err(
                        format!("Unknown mtree section '{}' found in '{}'.", x, line).into(),
                    )
                }
            }
        }
        if let Some(filepath) = filepath {
            ret.push(MTreeEntry {
                filepath,
                hashes,
                mode,
                gid,
                uid,
                time,
                filesize,
                filetype,
                link,
            });
        }
    }

    Ok(ret)
}

#[cfg(test)]
mod test {
    use crate::Result;

    #[test]
    fn test_mtree() -> Result<()> {
        let v = super::read_mtree_from_file("/var/lib/pacman/local/linux-5.11.6.arch1-1/mtree")?;
        println!("{:#?}", v);
        Ok(())
    }
}
