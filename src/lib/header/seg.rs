use crate::{FixedSize, Pup, SegmentId, SignatureKind};

use std::convert::{TryFrom, TryInto as _};

impl From<&Pup> for super::Table<Entry> {
    fn from(pup: &Pup) -> Self {
        // This is a usize (and not a u64) so that overflow is less likely. In practice, u64 is
        // the widest scalar type, so this doesn't really matter.
        let mut offset = pup.header_size();

        let entries = pup
            .segments
            .iter()
            .map(|seg| {
                let entry = Entry {
                    id: seg.id,
                    offset: offset as u64,
                    size: seg.data.len() as u64,
                    sig_kind: seg.sig_kind,
                };

                // [may_panic(Add)]
                offset += entry.size as usize;

                entry
            })
            .collect();

        Self(entries)
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

impl FixedSize for Entry {
    const SIZE: usize = 0x20;
}
