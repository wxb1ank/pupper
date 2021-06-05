use pupper::Pup;

pub fn execute(args: &clap::ArgMatches) -> Result<(), String> {
    let path = std::path::Path::new(args.value_of("pup").unwrap());
    let img_version = parse_img_version_option(args.value_of("img_version"))?;

    super::write_pup_to_path(&create_pup(img_version), path)
}

fn parse_img_version_option(img_version: Option<&str>) -> Result<u64, String> {
    img_version.map_or(Ok(0), |img_version| {
        img_version
            .parse()
            .map_err(|err| format!("failed to parse image version: {}", err))
    })
}

fn create_pup(img_version: u64) -> Pup {
    let mut pup = Pup::new();
    pup.image_version = img_version;

    pup
}
