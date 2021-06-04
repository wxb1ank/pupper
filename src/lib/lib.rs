#![feature(const_evaluatable_checked, const_generics)]

mod header;

use std::convert::{TryFrom, TryInto as _};

/// A PS3 [PUP] (Playstation Update Package).
///
/// [PUP]: https://www.psdevwiki.com/ps3/Playstation_Update_Package_(PUP)
#[derive(Clone, Debug, Default, Hash)]
pub struct Pup {
    /// The segments, or files, contained in the PUP.
    pub segments: Vec<Segment>,

    /// The [image version] of the PUP.
    ///
    /// [image version]: https://www.psdevwiki.com/ps3/Playstation_Update_Package_(PUP)#Header
    pub image_version: u64,
}

impl TryFrom<&[u8]> for Pup {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let header: header::Header = data.try_into()?;

        // First (and most importantly), we generate segments, drawing from three separate
        // locations: the segment table, the digest table, and the actual data.
        let segments = header
            .seg_table
            .iter()
            .enumerate()
            .flat_map(|(i, entry)| {
                let i = i as u64;

                let digest = header
                    .digest_table
                    .iter()
                    .find(|x| x.seg_index == i)
                    .ok_or(Self::Error::MissingDigest(i))
                    .map(|x| x.digest)?;

                let data = {
                    // TODO: Should non-wrapping addition be used here? Maybe it should even be
                    // saturating?
                    let start = entry.offset as usize;
                    let end = start + (entry.size as usize);

                    data.get(start..end)
                        .ok_or(Self::Error::MissingData(i))
                        .map(|x| x.to_vec())?
                };

                let seg = Segment {
                    id: entry.id,
                    sig_kind: entry.sig_kind,
                    digest,
                    data,
                };

                Result::<Segment, Self::Error>::Ok(seg)
            })
            .collect();

        // Next, we copy over metadata that aren't inherently represented in the segments.
        Ok(Self {
            segments,
            image_version: header.meta.img_version,
        })
    }
}

impl From<&Pup> for Vec<u8> {
    fn from(pup: &Pup) -> Self {
        let mut data = Vec::new();

        data
    }
}

impl Pup {
    /// Allocates an empty [`Pup`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
pub enum Error {
    Undersized,
    InvalidMagic(Magic),
    UnsupportedPackageVersion(u64),
    InvalidSignatureKind(u32),
    MissingDigest(u64),
    MissingData(u64),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Undersized => write!(f, "PUP is too small"),
            Self::InvalidMagic(magic) => {
                let magic = std::str::from_utf8(&magic.0).unwrap_or_default();
                write!(f, "magic '{}' is invalid", magic)
            }
            Self::UnsupportedPackageVersion(version) => {
                write!(f, "package version '{}' is unsupported", version)
            }
            Self::InvalidSignatureKind(kind) => {
                write!(f, "signature kind '{}' is invalid", kind)
            }
            Self::MissingDigest(i) => write!(f, "digest for segment {} is missing", i),
            Self::MissingData(i) => write!(f, "data for segment {} is missing", i),
        }
    }
}

#[derive(Clone, Debug, Hash)]
pub struct Segment {
    pub id: SegmentId,
    pub sig_kind: SignatureKind,
    pub digest: Digest,
    pub data: Vec<u8>,
}

/// The ID of a [`Segment`]. Can *usually* be [translated to a file name].
///
/// [translated to a file name]:
///     https://www.psdevwiki.com/ps3/Playstation_Update_Package_(PUP)#Segment_Entry_IDs
#[derive(Clone, Copy, Debug, Hash)]
pub struct SegmentId(pub u64);

impl SegmentId {
    /// Translates this ID to a known file name.
    #[must_use]
    pub fn file_name(self) -> Option<String> {
        match self.0 {
            0x100 => Some("version.txt"),
            0x101 => Some("license.xml"),
            0x102 => Some("promo_flags.txt"),
            0x103 => Some("update_flags.txt"),
            0x104 => Some("patch_build.txt"),
            0x200 => Some("ps3swu.self"),
            0x201 => Some("vsh.tar"),
            0x202 => Some("dots.txt"),
            0x203 => Some("patch_data.pkg"),
            0x300 => Some("update_files.tar"),
            0x501 => Some("spkg_hdr.tar"),
            0x601 => Some("ps3swu2.self"),
            _ => None,
        }
        .map(String::from)
    }
}

/// The [kind] of a [`Segment`] signature.
///
/// [kind]: https://www.psdevwiki.com/ps3/Playstation_Update_Package_(PUP)#Segment_Table
#[derive(Clone, Copy, Debug, Hash)]
pub enum SignatureKind {
    /// The segment is signed with HMAC-SHA1.
    HmacSha1,
    /// The segment is signed with HMAC-SHA256.
    HmacSha256,
}

impl std::convert::TryFrom<u32> for SignatureKind {
    type Error = crate::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::HmacSha1),
            2 => Ok(Self::HmacSha256),
            _ => Err(Self::Error::InvalidSignatureKind(value)),
        }
    }
}

impl From<SignatureKind> for u32 {
    fn from(sig_kind: SignatureKind) -> Self {
        match sig_kind {
            SignatureKind::HmacSha1 => 0,
            SignatureKind::HmacSha256 => 2,
        }
    }
}

impl std::fmt::Display for SignatureKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::HmacSha1 => write!(f, "HMAC-SHA1"),
            Self::HmacSha256 => write!(f, "HMAC-SHA256"),
        }
    }
}

/// The [hash digest] of a [`Segment`]. Always HMAC-SHA1.
///
/// [hash digest]: https://www.psdevwiki.com/ps3/Playstation_Update_Package_(PUP)#Digest_Table
#[derive(Clone, Copy, Debug, Hash)]
pub struct Digest(pub [u8; Self::SIZE]);

impl Region for Digest {
    const SIZE: usize = 0x14;
}

/// The file magic of a PUP.
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub struct Magic(pub [u8; Self::SIZE]);

impl Default for Magic {
    fn default() -> Self {
        Self(*b"SCEUF\0\0")
    }
}

impl Region for Magic {
    const SIZE: usize = 0x07;
}

pub trait Region {
    const SIZE: usize;
}
