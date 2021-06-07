//! A Sony PlayStation 3 PUP (PlayStation Update Package) implementation.
//!
//! # Overview
//!
//! The PS3 receives software updates in a file format called 'PUP'. These packages are essentially
//!'flat' file systems: they contain individual files, or 'segments', but lack any hierarchical
//! structure.
//!
//! This crate facilitates the creation and (de)serialization of PUPs.
//!
//! # Examples
//!
//! Let's first create a new [`Pup`] and assign it an image version:
//!
//! ```
//! use pupper::{Pup, Segment};
//!
//! let segments = Vec::<Segment>::new();
//! let image_version: u64 = 0xAAAA_BBBB;
//!
//! let pup = Pup::new(segments.clone(), image_version);
//!
//! assert_eq!(segments, pup.segments);
//! assert_eq!(image_version, pup.image_version);
//! ```
//!
//! As you can see, [`Pup`] is, for most intents and purposes, a [POD] type. [`Pup::image_version`]
//! is a public [`u64`], and [`Pup::segments`] is transparently a [`Vec<Segment>`].
//!
//! Let's now create a segment and add it to the [`Pup`] we previously created:
//!
//! ```no_run
//! use pupper::{Segment, SegmentId, SignatureKind};
//! #
//! # let mut pup = pupper::Pup::default(); // We can cheat a little here, LOL.
//!
//! let id = SegmentId(0x100);
//! let sig_kind = SignatureKind::HmacSha1;
//! let data = std::fs::read("foo.txt").unwrap();
//!
//! let segment = Segment::new(id, sig_kind, data.clone());
//!
//! // Segment is (mostly) a POD type, too!
//! assert_eq!(id, segment.id);
//! assert_eq!(sig_kind, segment.sig_kind);
//! assert_eq!(data, segment.data);
//!
//! pup.segments.push(segment.clone());
//! assert_eq!(segment, pup.segments[0]);
//! ```
//!
//! Finally, let's serialize the entire [`Pup`]. Afterwards, we'll deserialize it to confirm the
//! conversions were lossless:
//!
//! ```
//! use pupper::Pup;
//! use std::convert::TryFrom as _;
//! #
//! # let pup = Pup::default();
//!
//! // Serialize the PUP.
//! let data = Vec::<u8>::from(&pup);
//!
//! // Deserialize the PUP.
//! assert_eq!(Ok(pup), Pup::try_from(data.as_slice()));
//! ```
//!
//! [POD]: https://en.wikipedia.org/wiki/Passive_data_structure

// TODO: Remove these when they are stabilized OR suitable alternatives are found.
#![feature(const_evaluatable_checked, const_generics)]

mod header;

use header::Header;

use std::{
    convert::{TryFrom, TryInto as _},
    fmt::{self, Display, Formatter},
};

/// A PS3 PUP (PlayStation Update Package).
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Pup {
    /// The segments, or files, contained in this PUP.
    pub segments: Vec<Segment>,

    /// The image version of this PUP.
    ///
    /// Presumably, this field identifies the revision of this PUP's contents. I don't work for
    /// Sony, though. ¯\_(ツ)_/¯
    pub image_version: u64,
}

impl TryFrom<&[u8]> for Pup {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let header: Header = data.try_into()?;

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
                    // [may_panic(Add)]
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
        // Create the header first to generate the segment table and location information.
        let header = Header::from(pup);

        let header_size = header.meta.header_size as usize;
        let data_size = header.meta.data_size as usize;

        // [may_panic(Add)]
        let mut data = Self::from(&header);
        data.resize_with(header_size + data_size, Default::default);

        // Fill in data according to the offset and size specified by the segment entries.
        // Note: This will crash and burn if Header::from() gets things wrong...
        for (i, entry) in header.seg_table.iter().enumerate() {
            // [may_panic(Add)]
            let start = entry.offset as usize;
            let end = start + (entry.size as usize);

            data[start..end].copy_from_slice(&pup.segments[i].data);
        }

        data
    }
}

impl Pup {
    #[must_use]
    pub fn new(segments: Vec<Segment>, image_version: u64) -> Self {
        Self {
            segments,
            image_version,
        }
    }

    // The following methods exist on Pup because, without them, Metadata::from() would need to be
    // called every time header or data size must be known.

    fn header_size(&self) -> usize {
        // With just the segment count, we can calculate exactly what the full header size should
        // be.

        let mut header_size: usize;

        // [may_panic(Add)]
        header_size = header::meta::Metadata::SIZE;
        header_size += self.segments.len() * header::seg::Entry::SIZE;
        header_size += self.segments.len() * header::digest::Entry::SIZE;
        header_size += Digest::SIZE;
        header_size += header_size % 0x10; // Round up to a multiple of 0x10.

        header_size
    }

    fn data_size(&self) -> usize {
        // [may_panic(Iterator::sum)]
        self.segments.iter().map(|x| x.data.len()).sum::<usize>()
    }
}

/// An erroneous result returned by [`Pup::try_from`].
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// The input data is too short.
    Undersized,
    /// The file magic is invalid.
    InvalidMagic(Magic),
    /// The package version is unsupported.
    UnsupportedPackageVersion(u64),
    /// A signature kind field has an invalid value.
    InvalidSignatureKind(u32),
    /// A segment at a specific index has no corresponding digest.
    MissingDigest(u64),
    /// A segment at a specific index has no corresponding data.
    MissingData(u64),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Undersized => write!(f, "input data is too short"),
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

/// An individual file contained in a [`Pup`].
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Segment {
    /// The ID of this segment.
    pub id: SegmentId,
    /// The kind of signature this segment has.
    pub sig_kind: SignatureKind,
    /// The actual data this segment represents.
    pub data: Vec<u8>,
    digest: Digest,
}

impl Segment {
    #[must_use]
    pub fn new(id: SegmentId, sig_kind: SignatureKind, data: Vec<u8>) -> Self {
        Self {
            id,
            sig_kind,
            data,
            digest: Digest::default(),
        }
    }

    #[must_use]
    pub fn digest(&self) -> &Digest {
        &self.digest
    }
}

/// The ID of a [`Segment`]. Can *usually* be [translated to a file name].
///
/// [translated to a file name]:
///     https://www.psdevwiki.com/ps3/Playstation_Update_Package_(PUP)#Segment_Entry_IDs
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SegmentId(pub u64);

impl TryFrom<SegmentId> for &'static str {
    type Error = String;

    fn try_from(id: SegmentId) -> Result<Self, Self::Error> {
        SEGMENT_ID_MAP
            .iter()
            .find(|(value, _)| *value == id.0)
            .map(|(_, file_name)| *file_name)
            .ok_or_else(|| format!("segment ID '{}' has no corresponding file name", id.0))
    }
}

impl TryFrom<&str> for SegmentId {
    type Error = String;

    fn try_from(file_name: &str) -> Result<Self, Self::Error> {
        SEGMENT_ID_MAP
            .iter()
            .find(|(_, value)| *value == file_name)
            .map(|(id, _)| Self(*id))
            .ok_or_else(|| format!("file name '{}' has no corresponding segment ID", file_name))
    }
}

// This u64 <=> &str map exists because strings (e.g., these file names) would be prone to
// accidental modification if repeated verbatim in the above two TryFrom implementations.
//
// This isn't a HashMap because...
//   a. I don't think static HashMaps are possible?
//   b. There's only 12 KV pairs.
static SEGMENT_ID_MAP: [(u64, &str); 12] = [
    (0x100, "version.txt"),
    (0x101, "license.xml"),
    (0x102, "promo_flags.txt"),
    (0x103, "update_flags.txt"),
    (0x104, "patch_build.txt"),
    (0x200, "ps3swu.self"),
    (0x201, "vsh.tar"),
    (0x202, "dots.txt"),
    (0x203, "patch_data.pkg"),
    (0x300, "update_files.tar"),
    (0x501, "spkg_hdr.tar"),
    (0x601, "ps3swu2.self"),
];

/// The kind of cryptographic signature a [`Segment`] has.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SignatureKind {
    /// The segment is signed with Sony's HMAC-SHA1.
    HmacSha1,
    /// The segment is signed with Sony's HMAC-SHA256.
    HmacSha256,
}

impl Default for SignatureKind {
    fn default() -> Self {
        Self::HmacSha1
    }
}

impl TryFrom<u32> for SignatureKind {
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

impl Display for SignatureKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::HmacSha1 => write!(f, "HMAC-SHA1"),
            Self::HmacSha256 => write!(f, "HMAC-SHA256"),
        }
    }
}

/// The hash digest of a [`Segment`]. Always signed with [`SignatureKind::HmacSha1`].
///
/// # Examples
///
/// [`Self::fmt`] formats this digest as a lowercase hexadecimal string:
///
/// ```
/// let mut digest = pupper::Digest::default();
/// for (i, byte) in digest.0.iter_mut().enumerate() {
///     let i = (i as u8) & 0b1111;
///
///     // Lo nibble
///     *byte = i;
///     // Hi nibble
///     *byte |= (i << 4);
/// }
///
/// let expected = "00112233445566778899aabbccddeeff00112233";
/// assert_eq!(expected, format!("{}", digest));
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Digest(pub [u8; Self::SIZE]);

impl Display for Digest {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let digest: String = self.0.iter().map(|x| format!("{:02x}", *x)).collect();

        write!(f, "{}", digest)
    }
}

impl FixedSize for Digest {
    const SIZE: usize = 0x14;
}

/// The file magic of a PUP. Always `SCEUF\0\0\0`.
///
/// This type exists solely for being the 'return value' of [`Error::InvalidMagic`].
///
/// # Examples
///
/// [`Self::default`] will always return the aforementioned value:
///
/// ```
/// assert_eq!(*b"SCEUF\0\0\0", pupper::Magic::default().0);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Magic(pub [u8; Self::SIZE]);

impl Default for Magic {
    fn default() -> Self {
        Self(*b"SCEUF\0\0\0")
    }
}

impl FixedSize for Magic {
    const SIZE: usize = 0x08;
}

/// Has a fixed, or constant, size.
///
/// This trait exists to reduce redundancy when writing newtypes of arrays (e.g., [`Digest`],
/// [`Magic`]).
pub trait FixedSize {
    const SIZE: usize;
}
