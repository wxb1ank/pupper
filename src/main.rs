#[macro_use]
extern crate clap;

mod create;
mod print;
mod seg;

use pupper::Pup;

use std::{convert::TryInto as _, fs, path::Path};

fn main() {
    let args = clap::clap_app!(pupper =>
        (version: clap::crate_version!())
        (about: clap::crate_description!())
        (@arg pup: -f --file +required +takes_value "PUP file path")
        (@subcommand create =>
            (about: "Creates an empty PUP")
            (@arg img_version: -g --image_version +takes_value "PUP image version (default: 0)")
        )
        (@subcommand print =>
            (about: "Prints a textual representation of a PUP")
        )
        (@subcommand segment =>
            (about: "Segment-related subcommands")
            (@arg index: -n --index +takes_value "Segment index (default: 0)")
            (@subcommand extract =>
                (about: "Extracts a segment from a PUP")
                (@arg seg: -s --segment +required +takes_value "Segment file path")
            )
            (@subcommand insert =>
                (about: "Inserts a segment into a PUP")
                (@arg seg: -s --segment +required +takes_value "Segment file path")
                (@arg id: -x --id +takes_value "Segment ID (default: 0)")
            )
            (@subcommand remove =>
                (about: "Removes a segment from a PUP")
            )
        )
    )
    .get_matches();

    let path = std::path::Path::new(args.value_of("pup").unwrap());

    let result = match args.subcommand() {
        ("print", Some(_)) => print::execute(path),
        ("create", Some(args)) => create::execute(path, args),
        ("segment", Some(args)) => seg::execute(path, args),
        _ => Ok(()),
    };

    if let Err(err) = result {
        println!("error: {}", err);
    }
}

fn read_pup_from_path(path: &Path) -> Result<Pup, String> {
    read_data_from_path(path).and_then(|x| {
        x.as_slice()
            .try_into()
            .map_err(|err| format!("failed to parse PUP at '{}': {}", path.display(), err))
    })
}

fn read_data_from_path(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|err| format!("failed to read from '{}': {}", path.display(), err))
}

fn write_pup_to_path(pup: &Pup, path: &Path) -> Result<(), String> {
    write_data_to_path(&Vec::<u8>::from(pup), path)
}

fn write_data_to_path(data: &[u8], path: &Path) -> Result<(), String> {
    fs::write(path, data).map_err(|err| format!("failed to write to '{}': {}", path.display(), err))
}
