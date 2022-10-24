pub trait CheckName {
    fn is_valid_name(&self, name: &str) -> bool { !self.get_reserved_terms().contains(&name) }
    fn get_reserved_terms(&self) -> &'static [&'static str];
}