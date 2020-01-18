use crate::signal::Slot1;
use crate::util::Ptr;
use crate::view::{ActionCtx, View};
use miniqt_sys::*;
use std::os::raw::c_int;
use veda::Revision;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CheckboxState {
    Unchecked,
    PartiallyChecked,
    Checked,
}

pub struct Checkbox {
    label: String,
    checkbox: Option<Ptr<QCheckBox>>,
    state_changed: Option<Slot1<'static, c_int>>,
}

impl Checkbox {
    pub fn new(label: impl Into<String>) -> Self {
        Checkbox {
            label: label.into(),
            checkbox: None,
            state_changed: None,
        }
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
        if let Some(checkbox) = self.checkbox {
            unsafe {
                let label = self.label.clone().into();
                QAbstractButton_setText(checkbox.upcast().as_ptr(), &label);
            }
        }
    }
}

impl View for Checkbox {
    type Action = CheckboxState;

    /*fn update(&mut self, rev: Revision<bool>) {
        assert!(self.checkbox.is_some(), "not mounted");

        let check_state = match *rev.data() {
            true => Qt_CheckState_Checked,
            false => Qt_CheckState_Unchecked,
        };

        unsafe { QCheckBox_setCheckState(self.checkbox.unwrap().as_ptr(), check_state) }
    }*/

    fn mount(&mut self, actx: ActionCtx<CheckboxState>) {
        let checkbox = Ptr::new(unsafe { QCheckBox_new() });
        self.checkbox.replace(checkbox);

        let mut state_changed = Slot1::new(move |i| {
            actx.emit(match i {
                0 => CheckboxState::Unchecked,
                1 => CheckboxState::PartiallyChecked,
                2 => CheckboxState::Checked,
                _ => panic!("unexpected value for checkbox state"),
            });
        });

        unsafe {
            let label = self.label.clone().into();
            QAbstractButton_setText(checkbox.upcast().as_ptr(), &label);
            state_changed.connect(checkbox, qt_signal!("stateChanged(int)"));
        }

        self.state_changed.replace(state_changed);
    }

    fn widget_ptr(&self) -> Option<Ptr<QWidget>> {
        self.checkbox.map(Ptr::upcast)
    }
}
