pub(crate) mod digest;
pub(crate) mod meta;
pub(crate) mod seg;
mod table;

use meta::Metadata;
use table::Table;

use crate::{Digest, Error, FixedSize, Pup};

use std::convert::{TryFrom, TryInto as _};

#[derive(Clone, Default)]
pub struct Header {
    pub meta: Metadata,
    pub seg_table: Table<seg::Entry>,
    pub digest_table: Table<digest::Entry>,
    header_digest: Digest,
}

impl Header {
    pub fn new(
        meta: Metadata,
        seg_table: Table<seg::Entry>,
        digest_table: Table<digest::Entry>,
    ) -> Self {
        Self {
            meta,
            seg_table,
            digest_table,
            header_digest: Digest::default(),
        }
    }

    pub fn header_digest(&self) -> &Digest {
        &self.header_digest
    }
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

impl From<&Pup> for Header {
    fn from(pup: &Pup) -> Self {
        Self::new(pup.into(), pup.into(), pup.into())
    }
}

impl From<&Header> for Vec<u8> {
    fn from(header: &Header) -> Self {
        let mut data = Self::new();

        data.append(&mut <[u8; Metadata::SIZE]>::from(header.meta).into());
        data.append(&mut Self::from(&header.seg_table));
        data.append(&mut Self::from(&header.digest_table));
        data.append(&mut header.header_digest.0.into());

        // Pad the header to the requested size.
        data.resize_with(header.meta.header_size as usize, Default::default);

        data
    }
}
