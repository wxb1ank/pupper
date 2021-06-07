pub fn execute(path: &std::path::Path, index: usize) -> Result<(), String> {
    super::modify_pup_at_path(path, |pup| {
        if !(0..pup.segments.len()).contains(&index) {
            return Err(format!("index '{}' is out-of-bounds", index));
        }

        pup.segments.remove(index);

        Ok(())
    })
}
