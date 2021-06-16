use crate::structure::instructions::{Expr, Instr};
use crate::structure::modules::{
    ExportDesc, FuncIdx, GlobalIdx, ImportDesc, Module, TypeIdx, TypedIdx,
};
use crate::structure::types::FuncType;
use crate::WasmError;

use super::func::FuncAddr;
use super::global::{GlobalAddr, GlobalInst};
use super::mem::{MemAddr,MemInst};
use super::stack::Stack;
use super::table::TableAddr;
use super::val::Val;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

pub type ExternalModule = HashMap<String, ExternalVal>;
pub type ImportObjects = HashMap<String, ExternalModule>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExternalVal {
    Func(FuncAddr),
    #[serde(skip)]
    Table(TableAddr),
    #[serde(skip)]
    Mem(MemAddr),
    Global(GlobalAddr),
}

impl ExternalVal {
    pub fn as_func(self) -> Option<FuncAddr> {
        if let ExternalVal::Func(x) = self {
            Some(x)
        } else {
            None
        }
    }

    pub fn unwrap_func(self) -> FuncAddr {
        /*let mut f = self.as_func().unwrap();
        f.refp();
        f*/
        self.as_func().unwrap()
    }

    pub fn as_table(self) -> Option<TableAddr> {
        if let ExternalVal::Table(x) = self {
            Some(x)
        } else {
            None
        }
    }

    pub fn unwrap_table(self) -> TableAddr {
        self.as_table().unwrap()
    }

    pub fn as_mem(self) -> Option<MemAddr> {
        if let ExternalVal::Mem(x) = self {
            Some(x)
        } else {
            None
        }
    }

    pub fn unwrap_mem(self) -> MemAddr {
        self.as_mem().unwrap()
    }

    pub fn as_global(self) -> Option<GlobalAddr> {
        if let ExternalVal::Global(x) = self {
            Some(x)
        } else {
            None
        }
    }

    pub fn unwrap_global(self) -> GlobalAddr {
        self.as_global().unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExportInst {
    name: String,
    value: ExternalVal,
}

pub trait TypedIdxAccess<Idx>
where
    Idx: TypedIdx,
    Self: std::ops::Index<usize>,
{
    fn get_idx(&self, idx: Idx) -> &Self::Output {
        &self[idx.to_idx()]
    }
}

impl TypedIdxAccess<TypeIdx> for Vec<FuncType> {}
impl TypedIdxAccess<FuncIdx> for Vec<FuncAddr> {}
impl TypedIdxAccess<GlobalIdx> for Vec<GlobalAddr> {}

#[derive(Debug, Clone)]
pub struct ModuleInst {
    pub types: Vec<FuncType>,     // Done
    pub funcs: Vec<FuncAddr>,     // Done
    pub table: Option<TableAddr>, // unnecessary?
    pub mem: Option<MemAddr>,     // unnecessary?
    pub globals: Vec<GlobalAddr>, // Done
    pub exports: Vec<ExportInst>, // Done
}

impl ModuleInst {
    pub fn new(
        module: &Module,
        imports_objects: ImportObjects,
    ) -> Result<Rc<ModuleInst>, WasmError> {
        let mut result = ModuleInst {
            types: module.types.clone(),
            funcs: Vec::new(),
            table: None,
            mem: None,
            globals: Vec::new(),
            exports: Vec::new(),
        };

        for import in &module.imports {
            let val = imports_objects
                .get(&import.module.0)
                .and_then(|module| module.get(&import.name.0))
                .cloned()
                .ok_or_else(|| WasmError::LinkError)?;
            match &import.desc {
                ImportDesc::Func(idx) => {
                    result.funcs.push(
                        val.as_func()
                            .filter(|func| func.type_().is_match(result.types.get_idx(*idx)))
                            /*.filter(|func| match result.addrs.funcs[func.0]{
                                FuncInst::RuntimeFunc{type_, ..}=>type_.clone(),
                                FuncInst::HostFunc{type_, ..}=>type_.clone(),}
                            .is_match(result.types.get_idx(*idx)))*/
                            .ok_or_else(|| WasmError::LinkError)?,
                    );
                }
                ImportDesc::Table(type_) => {
                    let _ = result.table.replace(
                        val.as_table()
                            .filter(|table| table.type_().is_match(type_))
                            .ok_or_else(|| WasmError::LinkError)?,
                    );
                }
                ImportDesc::Mem(type_) => {
                    let _ = result.mem.replace(
                        val.as_mem()
                            .filter(|mem| mem.type_().is_match(type_))
                            .ok_or_else(|| WasmError::LinkError)?,
                    );
                }
                ImportDesc::Global(type_) => {
                    result.globals.push(
                        val.as_global()
                            .filter(|global| global.type_().is_match(type_))
                            .ok_or_else(|| WasmError::LinkError)?,
                    );
                }
            }
        }

        for _ in &module.funcs {
            result
                .funcs
                .push(FuncAddr::alloc_dummy(FuncIdx(result.funcs.len() as u32)));
        }

        if let Some(table) = module.tables.iter().next() {
            let _ = result.table.replace(TableAddr::alloc(&table.type_));
        }

        if let Some(mem) = module.mems.iter().next() {
            let _ = result.mem.replace(MemAddr::alloc(&mem.type_));
        }

        for global in &module.globals {
            result.globals.push(GlobalAddr::alloc(
                global.type_.clone(),
                result.eval_const_expr(&global.init),
                GlobalIdx(result.globals.len() as u32),
            ));
        }

        for elem in &module.elem {
            let offset = result.eval_const_expr(&elem.offset).unwrap_i32() as usize;
            result
                .table
                .as_ref()
                .unwrap()
                .instantiation_valid(offset, &elem.init)?;
        }
        for data in &module.data {
            let offset = result.eval_const_expr(&data.offset).unwrap_i32() as usize;
            result
                .mem
                .as_ref()
                .unwrap()
                .instantiation_valid(offset, &data.init.iter().map(|x| x.0).collect())?;
        }

        for elem in &module.elem {
            let offset = result.eval_const_expr(&elem.offset).unwrap_i32() as usize;
            result
                .table
                .as_ref()
                .unwrap()
                .init_elem(&result.funcs, offset, &elem.init);
        }
        for data in &module.data {
            let offset = result.eval_const_expr(&data.offset).unwrap_i32() as usize;
            result
                .mem
                .as_ref()
                .unwrap()
                .init_data(offset, &data.init.iter().map(|x| x.0).collect());
        }

        for export in &module.exports {
            result.exports.push(ExportInst {
                name: export.name.0.clone(),
                value: match export.desc {
                    ExportDesc::Func(idx) => ExternalVal::Func(result.funcs.get_idx(idx).clone()),
                    ExportDesc::Global(idx) => {
                        ExternalVal::Global(result.globals.get_idx(idx).clone())
                    }
                    ExportDesc::Mem(_idx) => ExternalVal::Mem(result.mem.as_ref().unwrap().clone()),
                    ExportDesc::Table(_idx) => {
                        ExternalVal::Table(result.table.as_ref().unwrap().clone())
                    }
                },
            });
        }

        let result = Rc::new(result);

        for (i, func) in module.funcs.iter().enumerate() {
            let idx = i + module
                .imports
                .iter()
                .map(|x| {
                    if let ImportDesc::Func(_) = x.desc {
                        1
                    } else {
                        0
                    }
                })
                .sum::<usize>();
            result.funcs[idx].replace_dummy(func.clone(), Rc::downgrade(&result));
        }

        if let Some(start) = &module.start {
            result
                .funcs
                .get_idx(start.func)
                .call(vec![], Rc::downgrade(&result))?;
        }

        Ok(result)
    }

    pub fn restore_mem(&self, json_str: String) {
        let mem: MemInst = serde_json::from_str(json_str.as_str()).unwrap();
        if let Some(memaddr) = &self.mem{
            memaddr.mut_inst().set(mem);
        }
    }

    pub fn restore_globals(&self, json_str: String) {
        let globals: Vec<GlobalInst> = serde_json::from_str(json_str.as_str()).unwrap();

        for i in 0..self.globals.len() {
            let global = globals[i];
            self.globals
                .get_idx(GlobalIdx(i as u32))
                .mut_inst()
                .set(global.value, global.mut_); //=globals.get_idx(GlobalIdx(i as u32));
        }
    }

    pub fn restore_stack(&self, json_str: String, instance: &Rc<ModuleInst>) -> Result<Option<Val>, WasmError> {
        let mut stack: Stack = serde_json::from_str(json_str.as_str()).unwrap();
        stack.module = Rc::downgrade(instance);
        //println!("restoreMod: {:?}", stack.module.upgrade().unwrap());

        let mut count =0;
        loop {
            stack.step(count)?;
            if stack.stack.len() == 1
                && stack.stack.first().unwrap().stack.len() == 1
                && stack
                    .stack
                    .first()
                    .unwrap()
                    .stack
                    .first()
                    .unwrap()
                    .instrs
                    .is_empty()
            {
                break;
            }
            count+=1;
            
        }
        println!("{}",count);
        Ok(stack.stack.pop().unwrap().stack.pop().unwrap().stack.pop())
    }

    fn eval_const_expr(&self, expr: &Expr) -> Val {
        match &expr.0[..] {
            &[Instr::I32Const(x)] => Val::I32(x),
            &[Instr::I64Const(x)] => Val::I64(x),
            &[Instr::F32Const(x)] => Val::F32(x),
            &[Instr::F64Const(x)] => Val::F64(x),
            &[Instr::GlobalGet(i)] => self.globals[i.to_idx()].get(),
            _ => panic!(),
        }
    }

    pub fn export(&self, name: &str) -> ExternalVal {
        self.exports
            .iter()
            .find(|e| e.name.as_str() == name)
            .map(|x| x.value.clone())
            .unwrap()
    }

    pub fn exports(&self) -> ExternalModule {
        self.exports
            .iter()
            .map(|x| (x.name.clone(), x.value.clone()))
            .collect()
    }
}
