use crate::{Digest, Magic, Pup, Region};

use std::convert::{TryFrom, TryInto as _};

#[derive(Clone, Copy, Default)]
pub struct Metadata {
    pub img_version: u64,
    pub seg_count: u64,
    pub header_size: u64,
    pub data_size: u64,
}

impl TryFrom<&[u8; Self::SIZE]> for Metadata {
    type Error = crate::Error;

    fn try_from(data: &[u8; Self::SIZE]) -> Result<Self, Self::Error> {
        // First, check the magic to verify this is actually a PUP file.
        let magic = Magic(data[0x00..0x08].try_into().unwrap());
        if magic != Magic::default() {
            return Err(Self::Error::InvalidMagic(magic));
        }

        // Next, check the package version to verify this PUP file is supported.
        let pkg_version = u64::from_be_bytes(data[0x08..0x10].try_into().unwrap());
        if pkg_version != Self::PKG_VERSION {
            return Err(Self::Error::UnsupportedPackageVersion(pkg_version));
        }

        // LGTM.
        let img_version = u64::from_be_bytes(data[0x10..0x18].try_into().unwrap());
        let seg_count = u64::from_be_bytes(data[0x18..0x20].try_into().unwrap());
        let header_size = u64::from_be_bytes(data[0x20..0x28].try_into().unwrap());
        let data_size = u64::from_be_bytes(data[0x28..0x30].try_into().unwrap());

        Ok(Self {
            img_version,
            seg_count,
            header_size,
            data_size,
        })
    }
}

impl From<&Pup> for Metadata {
    fn from(pup: &Pup) -> Self {
        // With just the segment count, we can calculate exactly what the full header size should
        // be.
        let mut header_size = Self::SIZE;
        header_size += pup.segments.len() * super::seg::Entry::SIZE;
        header_size += pup.segments.len() * super::digest::Entry::SIZE;
        header_size += Digest::SIZE;
        header_size += header_size % 0x10; // Round up to a multiple of 0x10.
        let header_size = header_size as u64;

         // Note: The usage of Iterator::sum() here may panic in case of an overflow.
        let data_size = pup.segments
            .iter()
            .map(|x| x.data.len())
            .sum::<usize>();
        let data_size = data_size as u64;

        Self {
            img_version: pup.image_version,
            seg_count: pup.segments.len() as u64,
            header_size,
            data_size,
        }
    }
}

impl From<Metadata> for [u8; Metadata::SIZE] {
    fn from(meta: Metadata) -> Self {
        let mut data = [0; Metadata::SIZE];

        data[0x00..0x08].copy_from_slice(&Magic::default().0);
        data[0x08..0x10].copy_from_slice(&Metadata::PKG_VERSION.to_be_bytes());
        data[0x10..0x18].copy_from_slice(&meta.img_version.to_be_bytes());
        data[0x18..0x20].copy_from_slice(&meta.seg_count.to_be_bytes());
        data[0x20..0x28].copy_from_slice(&meta.header_size.to_be_bytes());
        data[0x28..0x30].copy_from_slice(&meta.data_size.to_be_bytes());

        data
    }
}

impl Region for Metadata {
    const SIZE: usize = 0x30;
}

impl Metadata {
    const PKG_VERSION: u64 = 1;
}
