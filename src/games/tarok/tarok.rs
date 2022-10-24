use crate::core::traits::CheckName;

pub struct Tarok;

impl CheckName for Tarok {
    fn get_reserved_terms(&self) -> &'static [&'static str] {
        &["3","2", "1", "S3", "S2", "S1"]
    }
}
