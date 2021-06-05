#[macro_use]
extern crate clap;

use pupper::{Pup, Segment, SegmentId, SignatureKind};

use clap::ArgMatches;
use tracing::error;

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

    match args.subcommand() {
        ("print", Some(args)) => do_print(args),
        ("create", Some(args)) => do_create(args),
        ("insert", Some(args)) => do_insert(args),
        ("remove", Some(args)) => do_remove(args),
        // ("unpack", Some(args)) => do_unpack(args),
        _ => (),
    };
}

fn do_print(args: &ArgMatches) {
    let pup_path = Path::new(args.value_of("pup").unwrap());

    match read_pup_at_path(pup_path) {
        Ok(ref pup) =>  {
            println!("Image version: {:#x}", pup.image_version);
            println!("[Segments]");

            for seg in &pup.segments {
                let name = seg.id
                    .file_name()
                    .unwrap_or_else(|| format!("ID: {:#x}", seg.id.0));

                println!("  [{}]", name);
                println!("    Size: {} bytes", seg.data.len());

                let digest: String = seg.digest()
                    .0
                    .iter()
                    .map(|x| format!("{:02x}", x))
                    .collect();

                println!("    Hash digest: {} ({})", digest, seg.sig_kind);
            }
        }
        Err(err) => error!("{}", err),
    };
}

fn do_create(args: &ArgMatches) {
    let pup_path = Path::new(args.value_of("pup").unwrap());
    let img_version = args.value_of("img_version")
        .unwrap_or("0")
        .parse()
        .expect("invalid image version");

    let mut pup = Pup::new();
    pup.image_version = img_version;

    if let Err(err) = write_pup_at_path(pup_path, &pup) {
        error!("{}", err);
    }
}

fn do_insert(args: &ArgMatches) {
    let pup_path = Path::new(args.value_of("pup").unwrap());
    let seg_path = Path::new(args.value_of("seg").unwrap());

    let id: u64 = args.value_of("id")
        .unwrap_or("0")
        .parse()
        .expect("invalid segment ID");

    let mut index: usize = args.value_of("index")
        .unwrap_or("0")
        .parse()
        .expect("invalid segment index");

    match read_pup_at_path(pup_path) {
        Ok(ref mut pup) => match read_file_at_path(seg_path) {
            Ok(seg) => {
                let seg = Segment::new(
                    SegmentId(id),
                    SignatureKind::default(),
                    seg);

                if index > pup.segments.len() {
                    index = pup.segments.len();
                }

                pup.segments.insert(index, seg);

                if let Err(err) = write_pup_at_path(pup_path, pup) {
                    error!("{}", err);
                }
            }
            Err(err) => error!("{}", err),
        }
        Err(err) => error!("{}", err),
    }
}

fn do_remove(args: &ArgMatches) {
    let pup_path = Path::new(args.value_of("pup").unwrap());
    let index: usize = args.value_of("index")
        .unwrap_or("0")
        .parse()
        .expect("invalid segment index");


    println!("TODO"); // TODO
}

fn read_pup_at_path(path: &Path) -> Result<Pup, String> {
    read_file_at_path(path)
        .and_then(|x| {
            x.as_slice()
                .try_into()
                .map_err(|err| format!("failed to parse PUP at '{}': {}", path.display(), err))
        })
}

fn read_file_at_path(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path)
        .map_err(|err| format!("failed to read from '{}': {}", path.display(), err))
}

fn write_pup_at_path(path: &Path, pup: &Pup) -> Result<(), String> {
    write_file_at_path(path, &Vec::<u8>::from(pup))
}

fn write_file_at_path(path: &Path, data: &[u8]) -> Result<(), String> {
    fs::write(path, data)
        .map_err(|err| format!("failed to write to '{}': {}", path.display(), err))
}

/*
fn derive_output_path_with_ext(input: &Path, ext: &str) -> PathBuf {
    let mut output: PathBuf = input.into();

    if output.extension().is_some() {
        assert!(output.set_extension(""));
    } else {
        assert!(output.set_extension(ext));
    }

    output
}

fn unpack_pup_into_dir(pup: &Pup, dir: &Path) {
    if let Err(err) = fs::create_dir_all(dir) {
        return error!("failed to create directory at '{}': {}", dir.display(), err);
    }

    for (i, seg) in pup.segments.iter().enumerate() {
        let file_name = seg
            .id
            .file_name()
            .unwrap_or_else(|| format!("seg-{}.bin", i));

        let path = dir.join(file_name);

        match fs::File::create(&path) {
            Ok(mut file) => {
                if let Err(err) = file.write_all(&seg.data) {
                    return error!(
                        "failed to write to file at '{}': {}",
                        path.display(),
                        err
                    );
                }
            }
            Err(err) => {
                return error!("failed to create file at '{}': {}", path.display(), err);
            }
        };
    }
}
*/
