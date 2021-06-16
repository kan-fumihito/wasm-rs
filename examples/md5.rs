use maplit::hashmap;
use std::cell::RefCell;
use std::cell::{Ref, RefMut};
use std::ffi::CString;
use std::rc::Rc;
use wasm_rs::binary::decode_module;
use wasm_rs::exec::{ModuleInst, Val};

fn main() {
    let module = decode_module(
        &std::fs::read("md5-bin/target/wasm32-unknown-unknown/debug/md5-bin.wasm").unwrap(),
    )
    .unwrap();
    let instance = ModuleInst::new(&module, hashmap! {}).unwrap();
    //let instance = Rc::new(instance);

    let input_bytes = CString::new("abc").unwrap().into_bytes();
    let input_ptr = instance
        .export("alloc")
        .unwrap_func()
        .call(
            vec![Val::I32(input_bytes.len() as i32)],
            Rc::downgrade(&instance),
        )
        .unwrap()
        .unwrap()
        .unwrap_i32();

    instance
        .export("memory")
        .unwrap_mem()
        .write_buffer(input_ptr, &input_bytes[..]);

    let output_ptr = instance
        .export("md5")
        .unwrap_func()
        .call(vec![Val::I32(input_ptr as i32)], Rc::downgrade(&instance))
        .unwrap()
        .unwrap()
        .unwrap_i32() as usize;

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
