use maplit::hashmap;
use std::cell::RefCell;
use std::cell::{Ref, RefMut};
use std::ffi::CString;
use std::fs;
use std::rc::Rc;
use wasm_rs::binary::decode_module;
use wasm_rs::exec::{ModuleInst, Val};

fn main() {
    let module = decode_module(
        &std::fs::read("md5-bin/target/wasm32-unknown-unknown/debug/md5-bin.wasm").unwrap(),
    )
    .unwrap();
    let instance = ModuleInst::new(&module, hashmap! {}).unwrap();

    let mem_json = fs::read("mem.json").unwrap();
    let globals_json = fs::read("globals.json").unwrap();
    let stack_json = fs::read("stack.json").unwrap();

    instance.restore_mem(String::from_utf8(mem_json).unwrap());
    instance.restore_globals(String::from_utf8(globals_json).unwrap());
    let output_ptr = instance
        .restore_stack(String::from_utf8(stack_json).unwrap(), &instance)
        .unwrap()
        .unwrap()
        .unwrap_i32() as usize;
    /*
    let output_ptr = instance
        .export("md5")
        .unwrap_func()
        .call(vec![Val::I32(input_ptr as i32)], Rc::downgrade(&instance))
        .unwrap()
        .unwrap()
        .unwrap_i32() as usize;
        */

    let mem = instance.export("memory").unwrap_mem();
    println!(
        "{}",
        CString::new(
            mem.into_iter()
                .skip(output_ptr)
                .take_while(|x| *x != 0)
                .collect::<Vec<_>>(),
        )
        .unwrap()
        .into_string()
        .unwrap()
    );
}
