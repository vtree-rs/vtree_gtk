use vtree_macros;

vtree_macros::define_nodes!{
    nodes {
        RootContext: mul Window,
        Window<::params::Window>: @Widget,
        Box<::params::Box>: mul @Widget,
        Button: Label,
        Label: mul Text,
    }
    groups {
        Widget: Box Button Label,
    }
}
