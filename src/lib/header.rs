pub(crate) mod sig;
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
    pub sig_table: Table<sig::Entry>,
    header_sig: Digest,
}

impl Header {
    pub fn new(
        meta: Metadata,
        seg_table: Table<seg::Entry>,
        sig_table: Table<sig::Entry>,
    ) -> Self {
        Self {
            meta,
            seg_table,
            sig_table,
            header_sig: Digest::default(),
        }
    }

    pub fn header_sig(&self) -> &Digest {
        &self.header_sig
    }
}

impl TryFrom<&[u8]> for Header {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let meta: Metadata = data
            .get(..Metadata::SIZE)
            .ok_or(Self::Error::Undersized)
            .and_then(|x| <&[u8; Metadata::SIZE]>::try_from(x).unwrap().try_into())?;
        let data = &data[Metadata::SIZE..];

        let seg_table_size = (meta.seg_count as usize) * seg::Entry::SIZE;
        let seg_table = data
            .get(..seg_table_size)
            .ok_or(Self::Error::Undersized)
            .and_then(|x| x.try_into())?;
        let data = &data[seg_table_size..];

        let sig_table_size = (meta.seg_count as usize) * sig::Entry::SIZE;
        let sig_table = data
            .get(..sig_table_size)
            .ok_or(Self::Error::Undersized)
            .and_then(|x| x.try_into())?;
        let data = &data[sig_table_size..];

        let header_sig = data
            .get(..Digest::SIZE)
            .ok_or(Self::Error::Undersized)
            .map(|x| <[u8; Digest::SIZE]>::try_from(x).unwrap())?;
        let header_sig = Digest(header_sig);

        Ok(Self {
            meta,
            seg_table,
            sig_table,
            header_sig,
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
        data.append(&mut Self::from(&header.sig_table));
        data.append(&mut header.header_sig.0.into());

        // Pad the header to the requested size.
        data.resize_with(header.meta.header_size as usize, Default::default);

        data
    }
}
