use pupper::Pup;

use std::convert::TryFrom as _;

pub fn execute(path: &std::path::Path) -> Result<(), String> {
    super::read_pup_from_path(path).map(|ref pup| print_pup(pup))
}

fn print_pup(pup: &Pup) {
    println!("{{");
    println!("  \"image-version\": {},", pup.image_version);
    println!("  \"segments\": [");

    for (i, seg) in pup.segments.iter().enumerate() {
        println!("    {{");
        println!("      \"id\": {},", seg.id.0);

        let file_name = <&'static str>::try_from(seg.id)
            .map_or_else(|_| "null".into(), |x| format!("\"{}\"", x));
        println!("      \"file-name\": {},", file_name);

        println!("      \"size\": {},", seg.data.len());
        println!("      \"signature\": \"{}\"", seg.signature());
        print!("    }}");

        if i == (pup.segments.len() - 1) {
            println!();
        } else {
            println!(",");
        }
    }

    println!("  ]");
    println!("}}");
}
