use crate::{Pup, Region, SegmentId, SignatureKind};

use std::convert::{TryFrom, TryInto as _};

impl From<&Pup> for super::Table<Entry> {
    fn from(pup: &Pup) -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy, Default)]
pub struct Entry {
    pub id: SegmentId,
    pub offset: u64,
    pub size: u64,
    pub sig_kind: SignatureKind,
}

impl TryFrom<&[u8; Self::SIZE]> for Entry {
    type Error = crate::Error;

    fn try_from(data: &[u8; Self::SIZE]) -> Result<Self, Self::Error> {
        if data.len() < Self::SIZE {
            return Err(Self::Error::Undersized);
        }

        let id = SegmentId(u64::from_be_bytes(data[0x00..0x08].try_into().unwrap()));
        let offset = u64::from_be_bytes(data[0x08..0x10].try_into().unwrap());
        let size = u64::from_be_bytes(data[0x10..0x18].try_into().unwrap());
        let sig_kind = u32::from_be_bytes(data[0x18..0x1C].try_into().unwrap()).try_into()?;

        Ok(Self {
            id,
            offset,
            size,
            sig_kind,
        })
    }
}

impl From<Entry> for [u8; Entry::SIZE] {
    fn from(entry: Entry) -> Self {
        let mut data = [0; Entry::SIZE];

        data[0x00..0x08].copy_from_slice(&entry.id.0.to_be_bytes());
        data[0x08..0x10].copy_from_slice(&entry.offset.to_be_bytes());
        data[0x10..0x18].copy_from_slice(&entry.size.to_be_bytes());
        data[0x18..0x1C].copy_from_slice(&u32::from(entry.sig_kind).to_be_bytes());

        data
    }
}

impl Region for Entry {
    const SIZE: usize = 0x20;
}
