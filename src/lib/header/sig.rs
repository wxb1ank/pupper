use crate::{Digest, FixedSize, Pup};

use std::convert::{TryFrom, TryInto as _};

impl From<&Pup> for super::Table<Entry> {
    fn from(pup: &Pup) -> Self {
        let entries = pup
            .segments
            .iter()
            .enumerate()
            .map(|(i, seg)| Entry {
                seg_index: i as u64,
                sig: *seg.signature(),
            })
            .collect();

        Self(entries)
    }
}

#[derive(Clone, Copy, Default)]
pub struct Entry {
    pub seg_index: u64,
    pub sig: Digest,
}

impl TryFrom<&[u8; Self::SIZE]> for Entry {
    type Error = crate::Error;

    fn try_from(data: &[u8; Self::SIZE]) -> Result<Self, Self::Error> {
        let seg_index = u64::from_be_bytes(data[0x00..0x08].try_into().unwrap());
        let sig = Digest(data[0x08..0x1C].try_into().unwrap());

        Ok(Self { seg_index, sig })
    }
}

impl From<Entry> for [u8; Entry::SIZE] {
    fn from(entry: Entry) -> Self {
        let mut data = [0; Entry::SIZE];

        data[0x00..0x08].copy_from_slice(&entry.seg_index.to_be_bytes());
        data[0x08..0x1C].copy_from_slice(&entry.sig.0);

        data
    }
}

impl FixedSize for Entry {
    const SIZE: usize = 0x20;
}
