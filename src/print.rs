use pupper::Pup;

use std::convert::TryFrom as _;

pub fn execute(args: &clap::ArgMatches) -> Result<(), String> {
    let path = std::path::Path::new(args.value_of("pup").unwrap());
    super::read_pup_from_path(path).map(|ref pup| print_pup(pup))
}

fn print_pup(pup: &Pup) {
    println!("Image version: {:#x}", pup.image_version);
    println!("[Segments]");

    for seg in &pup.segments {
        let name = <&'static str>::try_from(seg.id)
            .map_or_else(|_| format!("ID: {:#x}", seg.id.0), String::from);

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
