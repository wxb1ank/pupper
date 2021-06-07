use pupper::Pup;

pub fn execute(path: &std::path::Path, args: &clap::ArgMatches) -> Result<(), String> {
    let img_version = parse_img_version_option(args.value_of("img_version"))?;
    let pup = Pup::new(Vec::new(), img_version);

    super::write_pup_to_path(&pup, path)
}

fn parse_img_version_option(img_version: Option<&str>) -> Result<u64, String> {
    img_version.map_or(Ok(0), |img_version| {
        img_version
            .parse()
            .map_err(|err| format!("failed to parse image version: {}", err))
    })
}
