mod common;

use crate::common::*;
use testresult::TestResult;
use wasmer::Value;

#[test]
fn simple_i32_counter() -> TestResult {
    let wasm = parse_wat(
        r#"
(module
  (type (;0;) (func (result i32)))
  (func $entry (type 0) (result i32)
    (local i32)
    i32.const 0
    i32.const 0
    i32.load offset=64
    i32.const 1
    i32.add
    local.tee 0
    i32.store offset=64
    local.get 0)
  (memory (;0;) 1)
  (export "memory" (memory 0))
  (export "entry" (func $entry)))
            "#,
    )?;
    let wasm = wasm_submemory::rewrite(&wasm, SUBMEMORY_SIZE as i32)?;
    let mut vm = VM::new(&wasm)?;
    vm.set_memory_size(10 * SUBMEMORY_SIZE)?;
    for i in 1..=10 {
        for j in 0..10 {
            vm.call("set_base", &[Value::I32(j * SUBMEMORY_SIZE as i32)])?;
            assert_eq!(returned_int(&vm.call("entry", &[])?)?, i);
        }
    }
    Ok(())
}

#[test]
fn rust_i32_counter() -> TestResult {
    let wasm = include_bytes!("../wasm/rust_i32_counter.wasm");
    let wasm = wasm_submemory::rewrite(wasm, SUBMEMORY_SIZE as i32)?;
    let mut vm = VM::new(&wasm)?;
    vm.set_memory_size(10 * SUBMEMORY_SIZE)?;
    for i in 1..=10 {
        for j in 0..10 {
            vm.call("set_base", &[Value::I32(j * SUBMEMORY_SIZE as i32)])?;
            assert_eq!(returned_int(&vm.call("entry", &[])?)?, i);
        }
    }
    Ok(())
}

#[test]
fn c_i32_counter() -> TestResult {
    let wasm = include_bytes!("../wasm/c_i32_counter.wasm");
    let wasm = wasm_submemory::rewrite(wasm, SUBMEMORY_SIZE as i32)?;
    let mut vm = VM::new(&wasm)?;
    vm.set_memory_size(10 * SUBMEMORY_SIZE)?;
    for i in 1..=10 {
        for j in 0..10 {
            vm.call("set_base", &[Value::I32(j * SUBMEMORY_SIZE as i32)])?;
            assert_eq!(returned_int(&vm.call("entry", &[])?)?, i);
        }
    }
    Ok(())
}

#[test]
fn zig_i32_counter() -> TestResult {
    let wasm = include_bytes!("../wasm/zig_i32_counter.wasm");
    let wasm = wasm_submemory::rewrite(wasm, SUBMEMORY_SIZE as i32)?;
    let mut vm = VM::new(&wasm)?;
    vm.set_memory_size(10 * SUBMEMORY_SIZE)?;
    for i in 1..=10 {
        for j in 0..10 {
            vm.call("set_base", &[Value::I32(j * SUBMEMORY_SIZE as i32)])?;
            assert_eq!(returned_int(&vm.call("entry", &[])?)?, i);
        }
    }
    Ok(())
}
