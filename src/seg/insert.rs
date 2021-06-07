use pupper::{Segment, SignatureKind};

use std::path::Path;

pub fn execute(pup_path: &Path, index: usize, args: &clap::ArgMatches) -> Result<(), String> {
    let seg_path = Path::new(args.value_of("seg").unwrap());
    let id = super::parse_id_option(args.value_of("id"), seg_path)?;

    super::modify_pup_at_path(pup_path, |pup| {
        if !(0..=pup.segments.len()).contains(&index) {
            return Err(format!("index '{}' is out-of-bounds", index));
        }

        crate::read_data_from_path(seg_path).map(|data| {
            let seg = Segment::new(id, SignatureKind::default(), data);
            pup.segments.insert(index, seg);
        })
    })
}
