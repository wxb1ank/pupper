pub fn execute(args: &clap::ArgMatches) -> Result<(), String> {
    let path = std::path::Path::new(args.value_of("pup").unwrap());
    let mut index = super::parse_index_option(args.value_of("index"))?;

    super::modify_pup_at_path(path, |pup| {
        if pup.segments.is_empty() {
            return Err("PUP has no segments".into());
        }

        if index >= pup.segments.len() {
            index = pup.segments.len() - 1;
        }

        pup.segments.remove(index);

        Ok(())
    })
}
