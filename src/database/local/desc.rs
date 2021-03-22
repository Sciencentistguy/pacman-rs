use std::path::Path;

use crate::Result;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref SPLITTING_REGEX: Regex = Regex::new(r"%(\w+)%\n((?:.+\n)+)").unwrap();
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})"
    )
    .unwrap();
}

/// Represents the data from the `desc` file of a local database entry. This contains information
/// about the pacakge itself, not the files it owns.
#[derive(Debug)]
pub struct PackageDescription {
    pub name: String,
    pub version: String,
    pub pkgbase: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub arch: Option<Arch>,
    pub build_date: Option<u64>,
    pub install_date: Option<u64>,
    pub packager: Option<Packager>,
    pub size: Option<u64>,
    pub reason: Option<u8>, // This appears to always be 1. TODO make this an enum
    pub licences: Vec<String>,
    pub validation: Option<Validation>,
    pub replaces: Vec<String>,
    pub dependencies: Vec<String>,
    pub optional_dependencies: Vec<OptionalDependency>,
    pub provides: Vec<String>,
    pub groups: Vec<String>,
    pub conflicts: Vec<String>,
}

pub fn read_desc_from_file<P: AsRef<Path>>(filepath: P) -> Result<PackageDescription> {
    let desc = std::fs::read_to_string(filepath)?;
    parse_desc(desc.as_str())
}

fn parse_desc(desc: &str) -> Result<PackageDescription> {
    let mut name = None;
    let mut version = None;
    let mut pkgbase = None;
    let mut description = None;
    let mut url = None;
    let mut arch = None;
    let mut build_date = None;
    let mut install_date = None;
    let mut packager = None;
    let mut size = None;
    let mut reason = None;
    let mut licences = None;
    let mut validation = None;
    let mut replaces = None;
    let mut dependencies = None;
    let mut optional_dependencies = None;
    let mut provides = None;
    let mut groups = None;
    let mut conflicts = None;
    for captures in SPLITTING_REGEX.captures_iter(desc) {
        match &captures[1] {
            "NAME" => {
                name = captures.get(2).map(|x| x.as_str().trim().to_owned());
            }
            "VERSION" => {
                version = captures.get(2).map(|x| x.as_str().trim().to_owned());
            }
            "BASE" => {
                pkgbase = captures.get(2).map(|x| x.as_str().trim().to_owned());
            }
            "DESC" => {
                description = captures.get(2).map(|x| x.as_str().trim().to_owned());
            }
            "URL" => {
                url = captures.get(2).map(|x| x.as_str().trim().to_owned());
            }
            "ARCH" => {
                let tmp = captures.get(2).map(|x| match x.as_str().trim() {
                    "any" => Ok(Arch::Any),
                    "x86_64" => Ok(Arch::x86_64),
                    x => Err(format!("Unexpected architecture: '{}'", x)),
                });
                if let Some(Err(e)) = tmp {
                    return Err(e.into());
                } else {
                    arch = tmp.map(|x| x.unwrap());
                }
            }
            "BUILDDATE" => {
                build_date = captures.get(2).and_then(|x| x.as_str().trim().parse().ok());
            }
            "INSTALLDATE" => {
                install_date = captures.get(2).and_then(|x| x.as_str().trim().parse().ok());
            }
            "PACKAGER" => {
                packager = captures.get(2).and_then(|x| {
                    let x = x.as_str().trim();
                    if x == "Unknown pacakger" {
                        return None;
                    }
                    let name = x[..x.find('<').map(|x| x - 1).unwrap_or(x.len())]
                        .trim()
                        .to_owned();
                    let email = EMAIL_REGEX.find(x).map(|x| x.as_str().to_owned());
                    Some(Packager { name, email })
                });
            }
            "SIZE" => {
                size = captures.get(2).and_then(|x| x.as_str().trim().parse().ok());
            }
            "REASON" => {
                reason = captures.get(2).and_then(|x| x.as_str().trim().parse().ok());
            }
            "LICENSE" => {
                licences = captures.get(2).map(|x| {
                    x.as_str()
                        .trim()
                        .split('\n')
                        .map(|licence| licence.trim().to_owned())
                        .collect()
                })
            }
            "VALIDATION" => {
                let tmp = captures.get(2).map(|x| match x.as_str().trim() {
                    "pgp" => Ok(Validation::Pgp),
                    "none" => Ok(Validation::None),
                    x => Err(format!("Unexpected validation type '{}'", x)),
                });

                if let Some(Err(e)) = tmp {
                    return Err(e.into());
                } else {
                    validation = tmp.map(|x| x.unwrap());
                }
            }
            "REPLACES" => {
                replaces = captures.get(2).map(|x| {
                    x.as_str()
                        .trim()
                        .split('\n')
                        .map(|pkgname| pkgname.trim().to_owned())
                        .collect()
                });
            }
            "DEPENDS" => {
                dependencies = captures.get(2).map(|x| {
                    x.as_str()
                        .trim()
                        .split('\n')
                        .map(|pkgname| pkgname.trim().to_owned())
                        .collect()
                });
            }
            "OPTDEPENDS" => {
                optional_dependencies = captures.get(2).map(|x| {
                    x.as_str()
                        .trim()
                        .split('\n')
                        .map(|line| {
                            let mut it = line.split(':');
                            OptionalDependency {
                                package: it.next().map(|x| x.trim().to_owned()).unwrap(),
                                reason: it.next().map(|x| x.trim().to_owned()),
                            }
                        })
                        .collect()
                });
            }
            "PROVIDES" => {
                provides = captures.get(2).map(|x| {
                    x.as_str()
                        .trim()
                        .split('\n')
                        .map(|pkgname| pkgname.trim().to_owned())
                        .collect()
                });
            }
            "GROUPS" => {
                groups = captures.get(2).map(|x| {
                    x.as_str()
                        .trim()
                        .split('\n')
                        .map(|x| x.trim().to_owned())
                        .collect()
                })
            }
            "CONFLICTS" => {
                conflicts = captures.get(2).map(|x| {
                    x.as_str()
                        .trim()
                        .split('\n')
                        .map(|pkgname| pkgname.trim().to_owned())
                        .collect()
                });
            }

            ref x => {
                return Err(format!(
                    "Unknown section '{}' in desc file for '{}'",
                    x,
                    name.unwrap_or_else(|| "<name not found>".into())
                )
                .into())
            }
        }
    }
    Ok(PackageDescription {
        name: name.ok_or("Every package must have a name.")?,
        version: version.ok_or("Every package must have a version.")?,
        pkgbase,
        description,
        url,
        arch,
        build_date,
        install_date,
        packager,
        size,
        reason,
        licences: licences.unwrap_or_else(Vec::new),
        validation,
        replaces: replaces.unwrap_or_else(Vec::new),
        dependencies: dependencies.unwrap_or_else(Vec::new),
        optional_dependencies: optional_dependencies.unwrap_or_else(Vec::new),
        provides: provides.unwrap_or_else(Vec::new),
        groups: groups.unwrap_or_else(Vec::new),
        conflicts: conflicts.unwrap_or_else(Vec::new),
    })
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Arch {
    Any,
    x86_64,
}

#[derive(Debug)]
pub enum Validation {
    None,
    Pgp,
}

#[derive(Debug)]
pub struct Packager {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug)]
pub struct OptionalDependency {
    pub package: String,
    pub reason: Option<String>,
}

#[cfg(test)]
mod test {
    use crate::Result;

    #[test]
    fn test_read_desc() -> Result<()> {
        let v = super::read_desc_from_file("/var/lib/pacman/local/linux-5.11.6.arch1-1/desc")?;
        println!("{:#?}", v);
        Ok(())
    }
}
