use super::LoadableRegion;
use crate::{Magic, Region};

use std::convert::{TryFrom, TryInto as _};

pub struct Metadata {
    pub fmt_flag: bool,
    pub pkg_version: u64,
    pub img_version: u64,
    pub seg_count: u64,
    pub header_size: u64,
    pub data_size: u64,
}

impl LoadableRegion<'_> for Metadata {

}

impl TryFrom<&[u8; Self::SIZE]> for Metadata {
    type Error = crate::Error;

    fn try_from(data: &[u8; Self::SIZE]) -> Result<Self, Self::Error> {
        // First, check the magic to verify this is actually a PUP file.
        let magic = Magic(data[0x00..0x07].try_into().unwrap());
        if magic != Magic::default() {
            return Err(Self::Error::InvalidMagic(magic));
        }

        // Next, check the package version to verify this PUP file is supported.
        let pkg_version = u64::from_be_bytes(data[0x08..0x10].try_into().unwrap());
        if pkg_version != Self::PKG_VERSION {
            return Err(Self::Error::UnsupportedPackageVersion(pkg_version));
        }

        // LGTM.
        let fmt_flag = data[0x07] != 0;
        let img_version = u64::from_be_bytes(data[0x10..0x18].try_into().unwrap());
        let seg_count = u64::from_be_bytes(data[0x18..0x20].try_into().unwrap());
        let header_size = u64::from_be_bytes(data[0x20..0x28].try_into().unwrap());
        let data_size = u64::from_be_bytes(data[0x28..0x30].try_into().unwrap());

        Ok(Self {
            fmt_flag,
            pkg_version,
            img_version,
            seg_count,
            header_size,
            data_size,
        })
    }
}

impl Region for Metadata {
    const SIZE: usize = 0x30;
}

impl Metadata {
    const PKG_VERSION: u64 = 1;
}
