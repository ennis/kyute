use crate::signal::Slot;
use crate::util::Ptr;
use crate::view::{ActionCtx, View};
use miniqt_sys::*;
use veda::{Data, Revision};

pub struct Button {
    label: String,
    qbutton: Option<Ptr<QPushButton>>,
    actx: Option<ActionCtx<ButtonAction>>,
    clicked: Option<Slot<'static>>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ButtonAction {
    Clicked,
    Released,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Button {
        Button {
            label: label.into(),
            actx: None,
            qbutton: None,
            clicked: None,
        }
    }

    fn emit_clicked(&self) {
        self.actx.as_ref().map(|x| x.emit(ButtonAction::Clicked));
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
        if let Some(button) = self.qbutton {
            unsafe {
                let label = self.label.clone().into();
                QAbstractButton_setText(button.upcast().as_ptr(), &label);
            }
        }
    }
}

impl View for Button {
    type Action = ButtonAction;

    /*fn update(&mut self, rev: Revision<S>) {
        eprintln!("Button update {:?}", rev.address());
        assert!(self.qbutton.is_some(), "not mounted");
    }*/

    fn mount(&mut self, actx: ActionCtx<ButtonAction>) {
        self.actx = Some(actx.clone());

        let mut clicked = Slot::new(move || actx.emit(ButtonAction::Clicked));

        let button;
        let label = self.label.clone().into();

        unsafe {
            button = Ptr::new(QPushButton_new());
            QAbstractButton_setText(button.upcast().as_ptr(), &label);
            clicked.connect(button, qt_signal!("clicked()"));
        }

        self.qbutton.replace(button);
        self.clicked.replace(clicked);
    }

    fn widget_ptr(&self) -> Option<Ptr<QWidget>> {
        self.qbutton.map(Ptr::upcast)
    }
}
