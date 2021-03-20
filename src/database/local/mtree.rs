use std::io::prelude::*;
use std::path::Path;

use crate::Result;

#[derive(Debug)]
pub struct MTreeEntry {
    filepath: String,
    hashes: Hashes,
    mode: u16,
    gid: u32,
    uid: u32, // My system (arch linux) uses a uint32_t for uid_t
    time: u64,
    filesize: usize,
    filetype: FileType,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FileType {
    Directory,
    File,

    None,
}

#[derive(Debug, PartialEq, Eq)]
struct Hashes {
    md5: Option<String>,
    sha256: Option<String>,
}

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
        let mut time = 0;
        let mut filetype = FileType::None;

        for section in line.trim().split(' ').map(|x| x.trim()) {
            println!("starting section '{}'", section);
            if !section.contains('=') {
                if section.starts_with("/set") || section == "#mtree" {
                    continue;
                } else {
                    filepath = Some(section.to_owned());
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
                "type" => {
                    filetype = match second {
                        "file" => FileType::File,
                        "dir" => FileType::Directory,
                        _ => return Err(format!("Unknown filetype '{}'", second).into()),
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

                x => return Err(format!("Unknown mtree section '{}'.", x).into()),
            }
            println!("{}", section);
        }
        if filepath.is_some() {
            ret.push(MTreeEntry {
                filepath: filepath.unwrap(),
                hashes,
                mode,
                gid,
                uid,
                time,
                filesize,
                filetype,
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
