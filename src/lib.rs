#![feature(plugin)]
#![feature(proc_macro)]

extern crate vtree;
extern crate vtree_macros;
extern crate gtk;

pub mod nodes;
pub mod params;

use std::collections::hash_map;
use std::collections::HashMap;
use gtk::prelude::*;
use nodes::RootContext;
use nodes::groups::AllNodes;
use vtree::diff;

#[derive(Debug)]
pub struct Differ;

impl diff::Differ<InnerContext, AllNodes> for Differ {
    fn diff_added(
        &mut self,
        ctx: &mut diff::Context<InnerContext, AllNodes>,
        curr: &diff::PathFrame<AllNodes>,
    ) {
        let ctx = &mut ctx.ctx;
        AllNodes::visit_enter(curr, &mut |curr| {
            let widget = match curr.node() {
                &AllNodes::RootContext(_) => {
                    return;
                }
                &AllNodes::Window(nodes::Window { ref params, .. }) => {
                    let window = gtk::Window::new(gtk::WindowType::Toplevel);
                    window.set_title(params.title.as_ref());
                    match ctx.container_widget_by_path.entry(curr.to_path()) {
                        hash_map::Entry::Occupied(v) => {
                            panic!("Window `{}` already added", v.key())
                        }
                        hash_map::Entry::Vacant(v) => {
                            v.insert(ContainerWidget::Container(window.clone().upcast()));
                        }
                    }
                    curr.on_exit(move |_| window.show_all());
                    return;
                }
                &AllNodes::Button(ref node) => {
                    ContainerWidget::Container(gtk::Button::new().upcast())
                }
                &AllNodes::Label(ref node) => {
                    let text: String = node.children
                        .iter()
                        .filter_map(|(_, n)| match n {
                            &AllNodes::Text(ref t) => Some(t.as_ref()),
                            _ => None,
                        })
                        .collect();
                    let label = gtk::Label::new(text.as_ref());
                    ContainerWidget::Label(label.upcast())
                }
                &AllNodes::Text(_) => return,
                _ => unimplemented!(),
            };
            ctx.add_widget(curr, widget.clone());
        });
    }

    fn diff_removed(
        &mut self,
        ctx: &mut diff::Context<InnerContext, AllNodes>,
        last: &diff::PathFrame<AllNodes>,
    ) {
        if let &AllNodes::Text(_) = last.node() {
            // if let &ContainerWidget::Label(ref label) = ctx.get_parent_widget(last) {
            //     label
            // } else {
            //     unreachable!();
            // }
            return;
        }

        let ctx = &mut ctx.ctx;
        ctx.get_widget(last).clone().into_widget().destroy();
        AllNodes::visit_enter(last, &mut |curr| ctx.remove_widget(curr));
    }

    fn diff_params_changed(
        &mut self,
        ctx: &mut diff::Context<InnerContext, AllNodes>,
        curr: &diff::PathFrame<AllNodes>,
        last: &diff::PathFrame<AllNodes>,
    ) {
    }

    fn diff_reordered<I: Iterator<Item = (usize, usize)>>(
        &mut self,
        ctx: &mut diff::Context<InnerContext, AllNodes>,
        parent: &diff::PathFrame<AllNodes>,
        indices: I,
    ) {
    }
}

#[derive(Clone)]
enum ContainerWidget {
    Label(gtk::Label),
    Box(gtk::Box),
    Grid(gtk::Grid),
    Container(gtk::Container),
    Widget(gtk::Widget),
}

impl ContainerWidget {
    fn into_widget(self) -> gtk::Widget {
        use ContainerWidget::*;
        match self {
            Label(v) => v.upcast(),
            Box(v) => v.upcast(),
            Grid(v) => v.upcast(),
            Container(v) => v.upcast(),
            Widget(v) => v,
        }
    }
}

pub struct InnerContext {
    container_widget_by_path: HashMap<diff::Path, ContainerWidget>,
}

impl InnerContext {
    fn get_widget(&self, path: &diff::PathFrame<AllNodes>) -> &ContainerWidget {
        let path = path.to_path();
        self.container_widget_by_path
            .get(&path)
            .expect(format!("unable to find widget (path: `{}`)", path).as_ref())
    }

    fn get_parent_widget(&self, path: &diff::PathFrame<AllNodes>) -> &ContainerWidget {
        let path = path.parent().unwrap().to_path();
        self.container_widget_by_path.get(&path).expect(
            format!("unable to find parent widget (path: `{}`)", path).as_ref(),
        )
    }

    fn remove_widget(&mut self, path: &diff::PathFrame<AllNodes>) {
        self.container_widget_by_path.remove(&path.to_path());
    }

    fn add_widget(&mut self, fp: &diff::PathFrame<AllNodes>, w: ContainerWidget) {
        let path = fp.to_path();
        {
            match self.get_parent_widget(fp) {
                &ContainerWidget::Container(ref con) => con.add(&w.clone().into_widget()),
                _ => {}
            }
        }
        match self.container_widget_by_path.entry(path) {
            hash_map::Entry::Occupied(v) => panic!("Widget `{}` already added", v.key()),
            hash_map::Entry::Vacant(v) => {
                v.insert(w);
            }
        }
    }
}

pub struct Context {
    diff_ctx: diff::Context<InnerContext, AllNodes>,
    last: AllNodes,
}

impl Context {
    pub fn new(curr: RootContext) -> Context {
        if !gtk::is_initialized() {
            gtk::init().map_err(|_| "Error initializing GTK").unwrap();
        }

        let mut curr: AllNodes = curr.into();
        let mut diff_ctx = diff::Context::new(InnerContext {
            container_widget_by_path: HashMap::new(),
        });
        AllNodes::expand_widgets(&mut curr, None, &diff::SimplePathFrame::new());
        diff::Differ::diff_added(&mut Differ, &mut diff_ctx, &diff::PathFrame::new(&curr));
        Context {
            diff_ctx: diff_ctx,
            last: curr,
        }
    }

    pub fn update(&mut self, curr: RootContext) {
        let mut curr: AllNodes = curr.into();
        AllNodes::expand_widgets(&mut curr, Some(&mut self.last), &diff::SimplePathFrame::new());
        AllNodes::diff(
            &diff::PathFrame::new(&curr),
            &diff::PathFrame::new(&self.last),
            &mut self.diff_ctx,
            &mut Differ,
        );
        self.last = curr;
    }
}
