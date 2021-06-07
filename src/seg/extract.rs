use std::path::Path;

pub fn execute(pup_path: &Path, index: usize, args: &clap::ArgMatches) -> Result<(), String> {
    let seg_path = Path::new(args.value_of("seg").unwrap());

    crate::read_pup_from_path(pup_path).and_then(|pup| {
        pup.segments
            .get(index)
            .ok_or_else(|| format!("index '{}' is out-of-bounds", index))
            .and_then(|seg| crate::write_data_to_path(&seg.data, seg_path))
    })
}
