//
// wallpaper.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use anyhow::Error;
use clap::arg_enum;
use serde::{Deserialize, Serialize};

use crate::{CrystalFamily, Transform2};

#[derive(Clone, Serialize, Deserialize)]
pub struct WallpaperGroup<'a> {
    pub name: &'a str,
    pub family: CrystalFamily,
    pub wyckoff_str: Vec<&'a str>,
}

/// Defining one of the Crystallographic wallpaper groups.
///
/// This is the highest level description of the symmetry operations of a crystal structure.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallpaper {
    pub name: String,
    pub family: CrystalFamily,
}

impl Wallpaper {
    pub fn new(group: &WallpaperGroup) -> Wallpaper {
        Wallpaper {
            name: String::from(group.name),
            family: group.family,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WyckoffSite {
    pub letter: char,
    pub symmetries: Vec<Transform2>,
    pub num_rotations: u64,
    pub mirror_primary: bool,
    pub mirror_secondary: bool,
}

impl WyckoffSite {
    pub fn new(group: &WallpaperGroup) -> Result<WyckoffSite, Error> {
        let symmetries = group
            .wyckoff_str
            .iter()
            .map(|&a| Transform2::from_operations(a))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(WyckoffSite {
            letter: 'a',
            symmetries,
            num_rotations: 1,
            mirror_primary: false,
            mirror_secondary: false,
        })
    }
    pub fn multiplicity(&self) -> usize {
        self.symmetries.len()
    }

    pub fn degrees_of_freedom(&self) -> &[bool] {
        // TODO implement -> This is only required for the non-general Wyckoff sites since all the
        // general sites have 3 degrees-of-freedom.
        //
        // This will be checked as a method of the Transform struct.
        &[true, true, true]
    }
}

arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Debug, Serialize, Deserialize)]
    pub enum WallpaperGroups {
        p1,
        p2,
        p1m1,
        p1g1,
        p2mm,
        p2mg,
        p2gg,
    }
}

pub fn get_wallpaper_group<'a>(name: WallpaperGroups) -> Result<WallpaperGroup<'a>, Error> {
    match name {
        WallpaperGroups::p1 => Ok(WallpaperGroup {
            name: "p1",
            family: CrystalFamily::Monoclinic,
            wyckoff_str: vec!["x,y"],
        }),
        WallpaperGroups::p2 => Ok(WallpaperGroup {
            name: "p2",
            family: CrystalFamily::Monoclinic,
            wyckoff_str: vec!["x,y", "-x,-y"],
        }),
        WallpaperGroups::p1m1 => Ok(WallpaperGroup {
            name: "p1m1",
            family: CrystalFamily::Orthorhombic,
            wyckoff_str: vec!["x,y", "-x,y"],
        }),
        WallpaperGroups::p1g1 => Ok(WallpaperGroup {
            name: "p1m1",
            family: CrystalFamily::Orthorhombic,
            wyckoff_str: vec!["x,y", "-x,y+1/2"],
        }),
        WallpaperGroups::p2mm => Ok(WallpaperGroup {
            name: "p2mm",
            family: CrystalFamily::Orthorhombic,
            wyckoff_str: vec!["x,y", "-x,-y", "-x,y", "x,-y"],
        }),
        WallpaperGroups::p2mg => Ok(WallpaperGroup {
            name: "p2mg",
            family: CrystalFamily::Orthorhombic,
            wyckoff_str: vec!["x,y", "-x, -y", "-x+1/2, y", "x+1/2, -y"],
        }),
        WallpaperGroups::p2gg => Ok(WallpaperGroup {
            name: "p2gg",
            family: CrystalFamily::Orthorhombic,
            wyckoff_str: vec!["x,y", "-x, -y", "-x+1/2, y+1/2", "x+1/2, -y+1/2"],
        }),
    }
}

#[cfg(test)]
mod wyckoff_site_tests {
    use super::*;

    pub fn create_wyckoff() -> WyckoffSite {
        WyckoffSite {
            letter: 'a',
            symmetries: vec![Transform2::identity()],
            num_rotations: 1,
            mirror_primary: false,
            mirror_secondary: false,
        }
    }

    #[test]
    fn multiplicity() {
        let wyckoff = create_wyckoff();
        assert_eq!(wyckoff.multiplicity(), 1);
    }
}
