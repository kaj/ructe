#[allow(dead_code)]
pub struct StaticFile {
    pub content: &'static [u8],
    pub name: &'static str,
}

impl StaticFile {
    pub fn get(name: &str) -> Option<&'static Self> {
        if let Ok(pos) = STATICS.binary_search_by_key(&name, |s| s.name) {
            return Some(STATICS[pos]);
        } else {
            None
        }
    }
}
