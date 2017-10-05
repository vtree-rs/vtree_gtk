use vtree_macros::define_params;
use std::borrow::Cow;

#[derive(Debug)]
pub enum AllEvent {
}

define_params!{
    #[derive(Default, Debug, Clone, PartialEq)]
    pub struct Window {
        pub title: Cow<'static, str>,
    }
}

define_params!{
    #[derive(Default, Debug, Clone, PartialEq)]
    pub struct Box {
        pub vertical: bool,
        pub spacing: u32,
    }
}
