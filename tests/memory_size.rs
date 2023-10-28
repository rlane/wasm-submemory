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
    i32.const 0)
  (memory (;0;) 16)
  (export "memory" (memory 0))
  (export "entry" (func 0)))
  "#,
    )?;

    wasm_submemory::rewrite(&wasm, SUBMEMORY_SIZE)?;
    Ok(())
}

#[test]
fn denied() -> TestResult {
    let wasm = parse_wat(
        r#"
(module
  (type (;0;) (func (result i32)))
  (func (;0;) (type 0) (result i32)
    i32.const 0)
  (memory (;0;) 17)
  (export "memory" (memory 0))
  (export "entry" (func 0)))
  "#,
    )?;

    let ret = wasm_submemory::rewrite(&wasm, SUBMEMORY_SIZE);
    assert!(ret
        .unwrap_err()
        .to_string()
        .contains("larger than submemory size"));
    Ok(())
}
