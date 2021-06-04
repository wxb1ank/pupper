use super::LoadableRegion;
use crate::{Digest, Region};

use std::convert::{TryFrom, TryInto as _};

#[derive(Clone, Copy)]
pub struct Entry {
    pub seg_index: u64,
    pub digest: Digest,
}

impl LoadableRegion<'_> for Entry {}

impl TryFrom<&[u8; Self::SIZE]> for Entry {
    type Error = crate::Error;

    fn try_from(data: &[u8; Self::SIZE]) -> Result<Self, Self::Error> {
        let seg_index = u64::from_be_bytes(data[0x00..0x08].try_into().unwrap());
        let digest = Digest(data[0x08..0x1C].try_into().unwrap());

        Ok(Self { seg_index, digest })
    }
}

impl From<Entry> for [u8; Entry::SIZE] {
    fn from(entry: Entry) -> Self {
        let mut data = [0; Entry::SIZE];

        data[0x00..0x08].copy_from_slice(&entry.seg_index.to_be_bytes());
        data[0x08..0x1C].copy_from_slice(&entry.digest.0);

        data
    }
}

impl Region for Entry {
    const SIZE: usize = 0x20;
}
