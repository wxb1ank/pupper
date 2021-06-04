use super::LoadableRegion;
use crate::{Region, SegmentId, SignatureKind};

use std::convert::{TryFrom, TryInto as _};

pub struct Entry {
    pub id: SegmentId,
    pub offset: u64,
    pub size: u64,
    pub sig_kind: SignatureKind,
}

impl LoadableRegion<'_> for Entry {

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

impl Region for Entry {
    const SIZE: usize = 0x20;
}
