/// Rcのみ
use crate::structure::types::{GlobalType, Mut};
use crate::structure::modules::{GlobalIdx};

use super::val::Val;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::cell::{Ref, RefMut};
use std::rc::Rc;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default,Copy)]
pub struct GlobalInst {
    pub value: Val,
    pub mut_: Mut,
}

impl GlobalInst{
    pub fn set(&mut self, value:Val,mut_:Mut){
        self.value=value;
        self.mut_=mut_;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GlobalAddr(
    #[serde(skip)] Rc<RefCell<GlobalInst>>, 
    pub GlobalIdx,
);

impl GlobalAddr {
    pub fn mut_inst(&self) -> RefMut<GlobalInst> {
        self.0.borrow_mut()
    }

    pub fn inst(&self) -> Ref<GlobalInst> {
        self.0.borrow()
    }

    pub fn get_inst(&self) -> GlobalInst{
        GlobalInst{
            mut_:self.inst().mut_,
            value: self.inst().value,
        }
    }

    pub fn type_(&self) -> GlobalType {
        let inst = self.inst();
        GlobalType(inst.mut_.clone(), inst.value.val_type())
    }

    pub fn new(mut_: Mut, val: Val, idx:GlobalIdx) -> GlobalAddr {
        GlobalAddr(Rc::new(RefCell::new(GlobalInst { value: val, mut_ })),idx)
    }

    pub(super) fn alloc(type_: GlobalType, val: Val, idx: GlobalIdx) -> GlobalAddr {
        GlobalAddr::new(type_.0, val, idx)
    }

    pub fn get(&self) -> Val {
        self.inst().value
    }

    pub fn set(&self, val: Val) -> Option<()> {
        let mut inst = self.mut_inst();
        if inst.mut_ == Mut::Var && inst.value.val_type() == val.val_type() {
            inst.value = val;
            Some(())
        } else {
            None
        }
    }
}
