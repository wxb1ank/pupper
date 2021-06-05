#[macro_use]
extern crate clap;

mod create;
mod print;
mod seg;

use pupper::Pup;

use std::{convert::TryInto as _, fs, path::Path};

fn main() {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .init();

    let args = clap::clap_app!(pupper =>
        (version: clap::crate_version!())
        (about: clap::crate_description!())
        (@subcommand print =>
            (about: "Prints a textual representation of a PUP")
            (@arg pup: -f --file +required +takes_value "Input PUP file path")
        )
        (@subcommand create =>
            (about: "Creates an empty PUP")
            (@arg pup: -f --file +required +takes_value "Output PUP file path")
            (@arg img_version: -g --image_version +takes_value "PUP image version")
        )
        (@subcommand insert =>
            (about: "Inserts a segment into a PUP")
            (@arg pup: -f --file +required +takes_value "PUP file path")
            (@arg seg: -s --segment +required +takes_value "Segment file path")
            (@arg id: -x --id +takes_value "Segment ID (default: 0)")
            (@arg index: -n --index +takes_value "Segment index (default: 0)")
        )
        (@subcommand remove =>
            (about: "Removes a segment from a PUP")
            (@arg pup: -f --file +required +takes_value "PUP file path")
            (@arg index: -n --index +takes_value "Segment index (default: 0)")
        )
    ).get_matches();

    let result = match args.subcommand() {
        ("print", Some(args)) => print::execute(args),
        ("create", Some(args)) => create::execute(args),
        ("insert", Some(args)) => seg::insert::execute(args),
        ("remove", Some(args)) => seg::remove::execute(args),
        // ("unpack", Some(args)) => do_unpack(args),
        _ => Ok(()),
    };

    if let Err(err) = result {
        tracing::error!("{}", err);
    }
}

fn read_pup_from_path(path: &Path) -> Result<Pup, String> {
    read_data_from_path(path)
        .and_then(|x| {
            x.as_slice()
                .try_into()
                .map_err(|err| format!("failed to parse PUP at '{}': {}", path.display(), err))
        })
}

fn read_data_from_path(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path)
        .map_err(|err| format!("failed to read from '{}': {}", path.display(), err))
}

fn write_pup_to_path(pup: &Pup, path: &Path) -> Result<(), String> {
    write_data_to_path(&Vec::<u8>::from(pup), path)
}

fn write_data_to_path(data: &[u8], path: &Path) -> Result<(), String> {
    fs::write(path, data)
        .map_err(|err| format!("failed to write to '{}': {}", path.display(), err))
}
