pub fn vk(vks: &[String], s: String) -> Option<String> {
    if vks.contains(&s) {
        Some(s)
    } else {
        None
    }
}
