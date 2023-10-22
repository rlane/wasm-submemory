mod common;

use crate::common::*;
use testresult::TestResult;

#[test]
fn allowed() -> TestResult {
    let wasm = parse_wat(
        r#"
(module
  (type (;0;) (func (result i32)))
  (func (;0;) (type 0) (result i32)
    i32.const 20
    global.set 0
    global.get 0
    global.get 1
    i32.add)
  (memory (;0;) 1)
  (global (;0;) (mut i32) (i32.const 0))
  (global (;1;) i32 (i32.const 22))
  (export "memory" (memory 0))
  (export "entry" (func 0)))
  "#,
    )?;

    wasm_submemory::rewrite(&wasm, SUBMEMORY_SIZE as i32)?;
    Ok(())
}

#[test]
fn denied() -> TestResult {
    let wasm = parse_wat(
        r#"
(module
  (type (;0;) (func (result i32)))
  (func (;0;) (type 0) (result i32)
    i32.const 20
    global.set 0
    i32.const 22
    global.set 1
    global.get 0
    global.get 1
    i32.add)
  (memory (;0;) 1)
  (global (;0;) (mut i32) (i32.const 0))
  (global (;1;) (mut i32) (i32.const 0))
  (export "memory" (memory 0))
  (export "entry" (func 0)))
  "#,
    )?;

    assert!(wasm_submemory::rewrite(&wasm, SUBMEMORY_SIZE as i32).is_err());
    Ok(())
}
