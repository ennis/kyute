#![feature(specialization)]
use kyute::miniqt_sys::*;
use kyute::view::{ButtonAction, View, ViewExt};
use kyute::Application;
use std::marker::PhantomData;
use std::rc::Rc;
use veda::lens::LensIndexExt;
use veda::{Data, Database, Identifiable, Lens};

#[derive(Data, Clone, Debug)]
pub struct Node {
    name: String,
    description: String,
}

impl Identifiable for Node {
    type Id = String;

    fn id(&self) -> String {
        self.name.clone()
    }
}

#[derive(Data, Clone, Debug)]
pub struct Document {
    nodes: Vec<Node>,
}

#[test]
fn simple() {
    let m = Document { nodes: Vec::new() };

    let mut db = Database::new(m);

    use kyute::view as kyv;

    let root = kyv::Root::<Document, ()>::new(kyv::Lensed::new(
        Document::nodes,
        kyv::List::new(|id: String| {
            Box::new(kyv::VBox::new(vec![
                Box::new(kyv::Lensed::new(Node::name, kyv::Label::new())),
                Box::new(kyv::Lensed::new(Node::description, kyv::Label::new())),
            ]))
        }),
    ));

    db.add_watcher(root.clone());

    db.append(
        Document::nodes,
        Node {
            name: "node".to_string(),
            description: "desc".to_string(),
        },
    );

    db.replace(
        Document::nodes.index(0).compose(Node::description),
        "updated!".to_string(),
    );

    loop {
        root.run();
    }
}
