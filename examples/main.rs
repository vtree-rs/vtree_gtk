#![feature(plugin)]
#![feature(proc_macro)]

extern crate gtk;
extern crate vtree;
extern crate vtree_markup;
extern crate vtree_gtk;

use vtree_markup::markup;
use vtree_gtk::nodes as n;
use vtree_gtk::Context;

fn main() {
    let a = markup!{
        n::RootContext /
    };
    let mut ctx = Context::new(a);

    let mut i = 0;
    gtk::timeout_add(1_000, move || {
        let a = markup! {
            n::RootContext
                n::Window@"win1" title="First GTK+ Program"
                    n::Button
                        n::Label {"i: " (i.to_string())}
        };
        ctx.update(a);
        i += 1;
        gtk::Continue(true)
    });

    gtk::main();
}
