mod digest;
mod meta;
mod seg;
mod table;

use meta::Metadata;
use table::Table;

use crate::{Digest, Error, Region};

use std::convert::{TryFrom, TryInto as _};

pub struct Header {
    pub meta: Metadata,
    pub seg_table: Table<seg::Entry>,
    pub digest_table: Table<digest::Entry>,
    pub header_digest: Digest,
}

impl TryFrom<&[u8]> for Header {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // Metadata.
        let meta: Metadata = data
            .get(..Metadata::SIZE)
            .ok_or(Self::Error::Undersized)
            .and_then(|x| <&[u8; Metadata::SIZE]>::try_from(x).unwrap().try_into())?;
        let data = &data[Metadata::SIZE..];

        // Segment table.
        let seg_table_size = (meta.seg_count as usize) * seg::Entry::SIZE;
        let seg_table = data
            .get(..seg_table_size)
            .ok_or(Self::Error::Undersized)
            .and_then(|x| x.try_into())?;
        let data = &data[seg_table_size..];

        // Digest table.
        let digest_table_size = (meta.seg_count as usize) * digest::Entry::SIZE;
        let digest_table = data
            .get(..digest_table_size)
            .ok_or(Self::Error::Undersized)
            .and_then(|x| x.try_into())?;
        let data = &data[digest_table_size..];

        // Header digest.
        let header_digest = data
            .get(..Digest::SIZE)
            .ok_or(Self::Error::Undersized)
            .map(|x| <[u8; Digest::SIZE]>::try_from(x).unwrap())?;
        let header_digest = Digest(header_digest);

        Ok(Self {
            meta,
            seg_table,
            digest_table,
            header_digest,
        })
    }
}

impl From<&Header> for Vec<u8> {
    fn from(header: &Header) -> Self {
        let mut data = Vec::new();

        data
    }
}

pub trait LoadableRegion<'a>: Region + TryFrom<&'a [u8; Self::SIZE], Error = Error> + Copy
where
    [u8; Self::SIZE]: Sized + From<Self>,
{
}
