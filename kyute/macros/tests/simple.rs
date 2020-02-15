#![feature(proc_macro_hygiene)]
#![feature(specialization)]
use kyute_macros::view;
use kyute_macros::Data;

use kyute::view::VBox;

#[derive(Data, Clone, Debug)]
pub struct NodeInput {
    name: String,
}

#[derive(Data, Clone, Debug)]
pub struct NodeOutput {
    name: String,
}

#[derive(Data, Clone, Debug)]
pub struct Node {
    name: String,
    inputs: Vec<NodeInput>,
    outputs: Vec<NodeOutput>,
}

#[derive(Data, Clone, Debug)]
pub struct Document {
    counter: i32,
    nodes: Vec<Node>,
    connections: Vec<(i32, i32)>,
}

#[test]
fn test_view() {
    view! {
        pub ViewName(node: Node, label: String) -> VBox {
            [:label] Label {
                .text = label.clone();
                .alt_text = label.clone();
            }

            VBox {
                Label {
                    [:node][Node::name] .text = node.name();
                }
            }
        }

        // VBox<(Label,VBox<Label>)>

        // problem: must spell the VBox type (and the whole tree, for that matter)
        // -> boxing defeats the purpose, since we want to be able to access contents of a container
        //      without dynamic cast / any
        // -> no inference AT ALL
        //      e.g. can't have generic widgets whose types are inferred with the bound properties
        // -> only possible way is to move a Rc<ViewWithTheRealType> into a closure and use this closure
        //    for updates
        //
    };

    /*fn update_label(&mut self, label: Diff<String>) {
        // everything that falls under a [:label]

        // enter gen_update(label)
        // enter root
        //  - check guard
        // enter root items
        // for each item
        // - check guard
        // -

        let root = self.root;
        let l8 = root.contents().0;

        // unguarded expr
    }*/

    /*fn update_node(&mut self, node: Diff<Node>) {
        let root = self.root;
        let l13 = root.contents().0;
        let l14 = l13.contents().0;

        if let Some(_) = node.focus(Node::name) {
            l14.text().set(node.name());
        }
    }*/
}
