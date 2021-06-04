use pupper::Pup;
use yansi::Paint;

use std::{convert::TryInto as _, fs, io::Write as _, path::{Path, PathBuf}};

macro_rules! format_err {
    ($($arg:tt)*) => {
        format!("{} {}", Paint::red("error:"), format!($($arg)*))
    }
}

macro_rules! println_err {
    ($($arg:tt)*) => {
        println!("{}", format_err!($($arg)*))
    }
}

fn main() {
    // See <https://no-color.org/>.
    if std::env::var("NO_COLOR").is_ok() {
        Paint::disable();
    }

    let args: Vec<String> = std::env::args()
        .skip(1) // Skip program name.
        .collect();

    match args.split_first() {
        Some((command, args)) => do_command_with_args(command, args),
        None => print_usage(),
    };
}

fn do_command_with_args(command: &str, args: &[String]) {
    match command.to_ascii_lowercase().as_str() {
        "help" => print_usage(),
        "print" => {
            if args.is_empty() {
                return println_err!("'{}' is missing 'input' argument", command);
            }

            match read_pup_at_path(Path::new(&args[0])) {
                Ok(ref pup) => print_pup(pup),
                Err(err) => println_err!("{}", err),
            };
        }
        "extract" => {
            if args.is_empty() {
                return println_err!("'{}' is missing 'input' argument", command);
            }

            let pup_path = Path::new(&args[0]);

            let dir = if let Some(dir) = args.get(1) {
                PathBuf::from(dir)
            } else {
                let mut dir = PathBuf::from(pup_path);

                if dir.extension().is_some() {
                    // The original file path has an extension, so the output directory can
                    // simply be that same path without the extension.
                    assert!(dir.set_extension(""));
                } else {
                    // Let's add one instead.
                    assert!(dir.set_extension("out"));
                }

                dir
            };

            match read_pup_at_path(pup_path) {
                Ok(ref pup) => extract_pup_to_dir(pup, &dir),
                Err(err) => println_err!("{}", err),
            };
        }
        _ => println_err!("invalid command '{}'", command),
    }
}

fn print_usage() {
    println!("OVERVIEW: PS3 PUP utility");
    println!();
    println!("USAGE: {} <command> [arguments]", program_name());
    println!();
    println!("COMMANDS:");
    println!("  help                    Prints this help information.");
    println!("  print <input:PUP>       Prints a textual representation of <input>.");
    println!("  extract <input:PUP> [<output:DIR>]");
    println!("                          Extracts <input> to <output> (defaults to <input> without a file extension or, if <input> already has no extension, to <input>.out.");
}

fn program_name() -> String {
    std::env::args()
        .next()
        .unwrap_or_else(|| "pupper".into())
}

fn read_pup_at_path(path: &Path) -> Result<Pup, String> {
    match fs::read(path) {
        Ok(data) => {
            data.as_slice()
                .try_into()
                .map_err(|err| format!("failed to read from PUP: {}", err))
        }
        Err(err) => Err(format!("failed to read from '{}': {}", path.display(), err)),
    }
}

fn print_pup(pup: &Pup) {
    println!("Package version: {}", pup.package_version());
    println!("Image version:   {}", pup.image_version());

    for seg in &pup.segments {
        println!("--------------------");

        match seg.id.file_name() {
            Some(file_name) => println!("File name: {}", file_name),
            None => println!("ID: {:#x}", seg.id.0),
        };

        println!("Signature kind: {}", seg.sig_kind);
        println!("Size: {}", seg.data.len());

        let digest: String = seg.digest.0
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect();

        println!("Digest: {}", digest);
    }
}

fn extract_pup_to_dir(pup: &Pup, dir: &Path) {
    if let Err(err) = fs::create_dir_all(dir) {
        return println_err!("failed to create directory at '{}': {}", dir.display(), err);
    }

    for (i, seg) in pup.segments.iter().enumerate() {
        let file_name = seg.id
            .file_name()
            .unwrap_or_else(|| format!("seg-{}.bin", i));

        let path = dir.join(file_name);

        match fs::File::create(&path) {
            Ok(mut file) => if let Err(err) = file.write_all(&seg.data) {
                return println_err!("failed to write to file at '{}': {}", path.display(), err);
            }
            Err(err) => {
                return println_err!("failed to create file at '{}': {}", path.display(), err);
            }
        };
    }
}
