mod extract;
mod insert;
mod remove;

use pupper::{Pup, SegmentId};

use std::{convert::TryFrom as _, path::Path};

pub fn execute(path: &Path, args: &clap::ArgMatches) -> Result<(), String> {
    let index = parse_index_option(args.value_of("index"))?;

    match args.subcommand() {
        ("extract", Some(args)) => extract::execute(path, index, args),
        ("insert", Some(args)) => insert::execute(path, index, args),
        ("remove", Some(_)) => remove::execute(path, index),
        _ => Ok(()),
    }
}

fn parse_index_option(index: Option<&str>) -> Result<usize, String> {
    index.map_or(Ok(0), |index| {
        index
            .parse()
            .map_err(|err| format!("failed to parse segment index: {}", err))
    })
}

fn parse_id_option(id: Option<&str>, path: &Path) -> Result<SegmentId, String> {
    id.map_or_else(
        || {
            // Let's try to derive the segment ID from the file name.
            let file_name = path.file_stem();

            let id = file_name
                .and_then(std::ffi::OsStr::to_str)
                .map(SegmentId::try_from)
                .and_then(Result::ok)
                .unwrap_or_else(SegmentId::default);

            Ok(id)
        },
        |id| {
            id.parse()
                .map(SegmentId)
                .map_err(|err| format!("failed to parse segment ID: {}", err))
        },
    )
}

fn modify_pup_at_path<F>(path: &Path, f: F) -> Result<(), String>
where
    F: FnOnce(&mut Pup) -> Result<(), String>,
{
    super::read_pup_from_path(path).and_then(|ref mut pup| {
        f(pup)?;
        super::write_pup_to_path(pup, path)
    })
}
