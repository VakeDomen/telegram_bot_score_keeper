use crate::core::traits::CheckName;

pub struct Table;

impl CheckName for Table {
    fn get_reserved_terms(&self) -> &'static [&'static str] {
        &[]
    }
}