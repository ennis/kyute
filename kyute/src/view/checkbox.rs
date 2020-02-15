use crate::model::{Data, Lens, Revision};
use crate::signal::Slot1;
use crate::util::Ptr;
use crate::view::binding::Binding;
use crate::view::{ActionCtx, View};
use miniqt_sys::*;
use std::marker::PhantomData;
use std::os::raw::c_int;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CheckboxState {
    Unchecked,
    PartiallyChecked,
    Checked,
}

pub struct Checkbox<S: Data, Label: Lens<S, String>, State: Lens<S, CheckboxState>> {
    label: Label,
    state: State,
    checkbox: Option<Ptr<QCheckBox>>,
    state_changed: Option<Slot1<'static, c_int>>,
    _phantom: PhantomData<S>,
}

impl<S: Data, Label: Lens<S, String>, State: Lens<S, CheckboxState>> Checkbox<S, Label, State> {
    pub fn new(label: Label, state: State) -> Self {
        Checkbox {
            label,
            state,
            checkbox: None,
            state_changed: None,
            _phantom: PhantomData,
        }
    }
}

impl<S: Data, Label: Lens<S, String>, State: Lens<S, CheckboxState>> View<S>
    for Checkbox<S, Label, State>
{
    type Action = CheckboxState;

    fn update(&mut self, s: &Revision<S>) {
        let checkbox = self.checkbox.expect("not mounted");

        if let Some(label) = self.label.compute_if_changed(s) {
            unsafe {
                QAbstractButton_setText(checkbox.upcast().as_ptr(), &label.into());
            }
        }

        if let Some(s) = self.state.compute_if_changed(&s) {
            let check_state = match s {
                CheckboxState::Checked => Qt_CheckState_Checked,
                CheckboxState::Unchecked => Qt_CheckState_Checked,
                CheckboxState::PartiallyChecked => Qt_CheckState_PartiallyChecked,
            };

            unsafe { QCheckBox_setCheckState(checkbox.as_ptr(), check_state) }
        }
    }

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
            //let label = self.label.clone().into();
            //QAbstractButton_setText(checkbox.upcast().as_ptr(), &label);
            state_changed.connect(checkbox, qt_signal!("stateChanged(int)"));
        }

        self.state_changed.replace(state_changed);
    }

    fn widget_ptr(&self) -> Option<Ptr<QWidget>> {
        self.checkbox.map(Ptr::upcast)
    }
}
