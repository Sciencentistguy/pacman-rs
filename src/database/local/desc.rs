#![allow(dead_code)]

use core::panic;
use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref SPLITTING_REGEX: Regex = Regex::new(r"%(\w+)%\n((?:.+\n)+)").unwrap();
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})"
    )
    .unwrap();
}

#[derive(Debug)]
struct PackageDescription {
    name: Option<String>,
    version: Option<String>,
    pkgbase: Option<String>,
    description: Option<String>,
    url: Option<String>,
    arch: Option<Arch>,
    build_date: Option<u64>,
    install_date: Option<u64>,
    packager: Option<Packager>,
    size: Option<u64>,
    reason: Option<u8>, // This appears to always be 1
    licences: Option<Vec<String>>,
    validation: Option<Validation>,
    replaces: Option<Vec<String>>,
    dependencies: Option<Vec<String>>,
    optional_dependencies: Option<Vec<OptionalDependency>>,
    provides: Option<Vec<String>>,
}

impl PackageDescription {
    pub fn read_from_file<P: AsRef<Path>>(filepath: P) -> Result<Self, Box<dyn std::error::Error>> {
        let desc = std::fs::read_to_string(filepath)?;
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
        for captures in SPLITTING_REGEX.captures_iter(desc.as_str()) {
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
                    arch = captures.get(2).map(|x| match x.as_str().trim() {
                        "any" => Arch::Any,
                        "x86_64" => Arch::x86_64,
                        x => panic!("Unexpected architecture: '{}'", x),
                    });
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
                    validation = captures.get(2).map(|x| match x.as_str().trim() {
                        "pgp" => Validation::Pgp,
                        "none" => Validation::None,
                        x => panic!("Unexpected validation type '{}'", x),
                    })
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
                                    reason: it.next().map(|x| x.trim().to_owned()).unwrap(),
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

                ref x => panic!("Unknown desc section: '{}'", x),
            }
        }
        let splitted = desc.split('%');
        for thing in splitted {
            println!("{}", thing);
        }

        Ok(Self {
            name,
            version,
            pkgbase,
            description,
            url,
            arch,
            build_date,
            install_date,
            packager,
            size,
            reason,
            licences,
            validation,
            replaces,
            dependencies,
            optional_dependencies,
            provides,
        })
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
enum Arch {
    Any,
    x86_64,
}

#[derive(Debug)]
enum Validation {
    None,
    Pgp,
}

#[derive(Debug)]
struct Packager {
    name: String,
    email: Option<String>,
}

#[derive(Debug)]
struct OptionalDependency {
    package: String,
    reason: String,
}

#[cfg(test)]
mod test {
    #[test]
    fn test_read_desc() {
        let v = super::PackageDescription::read_from_file("/var/lib/pacman/local/linux-5.11.6.arch1-1/desc");
        assert!(v.is_ok());
        println!("{:#?}", v.unwrap());
    }
}
