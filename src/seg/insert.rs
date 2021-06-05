use pupper::{SignatureKind, Segment};

use std::path::Path;

pub fn execute(args: &clap::ArgMatches) -> Result<(), String> {
    let pup_path = Path::new(args.value_of("pup").unwrap());
    let seg_path = Path::new(args.value_of("seg").unwrap());

    let id = super::parse_id_option(args.value_of("id"), seg_path)?;
    let mut index = super::parse_index_option(args.value_of("index"))?;

    super::modify_pup_at_path(pup_path, |pup| {
        if index > pup.segments.len() {
            index = pup.segments.len();
        }

        crate::read_data_from_path(seg_path)
            .map(|data| {
                let seg = Segment::new(id, SignatureKind::default(), data);
                pup.segments.insert(index, seg);
            })
    })
}
