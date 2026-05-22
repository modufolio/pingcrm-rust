pub trait Factory {
    type Model;
    type NewModel;

    fn build(&self) -> Self::NewModel;

    fn table_name() -> &'static str;
}
