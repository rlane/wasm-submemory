mod common;

use crate::common::*;
use testresult::TestResult;
use wasmer::Value;

struct Testcase<'a> {
    name: &'a str,
    wasm: &'a [u8],
}

#[test]
fn i32_counter() -> TestResult {
    let wat_wasm = parse_wat(
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

    let testcases = &[
        Testcase {
            name: "wat",
            wasm: &wat_wasm,
        },
        Testcase {
            name: "rust",
            wasm: include_bytes!("../wasm/rust_i32_counter.wasm"),
        },
        Testcase {
            name: "c",
            wasm: include_bytes!("../wasm/c_i32_counter.wasm"),
        },
        Testcase {
            name: "zig",
            wasm: include_bytes!("../wasm/zig_i32_counter.wasm"),
        },
    ];

    for testcase in testcases {
        let wasm = wasm_submemory::rewrite(testcase.wasm, SUBMEMORY_SIZE as i32)?;
        let mut vm = VM::new(&wasm)?;
        vm.set_memory_size(10 * SUBMEMORY_SIZE)?;
        for i in 1..=10 {
            for j in 0..10 {
                vm.call("set_base", &[Value::I32(j * SUBMEMORY_SIZE as i32)])?;
                assert_eq!(
                    returned_int(&vm.call("entry", &[])?)?,
                    i,
                    "{}",
                    testcase.name
                );
            }
        }
    }

    Ok(())
}

#[test]
fn f32_counter() -> TestResult {
    let wat_wasm = parse_wat(
        r#"
(module
  (type (;0;) (func (result f32)))
  (func $entry (type 0) (result f32)
    (local f32)
    i32.const 0
    i32.const 0
    f32.load offset=64
    f32.const 1
    f32.add
    local.tee 0
    f32.store offset=64
    local.get 0)
  (memory (;0;) 1)
  (export "memory" (memory 0))
  (export "entry" (func $entry)))
            "#,
    )?;

    let testcases = &[
        Testcase {
            name: "wat",
            wasm: &wat_wasm,
        },
        Testcase {
            name: "rust",
            wasm: include_bytes!("../wasm/rust_f32_counter.wasm"),
        },
        Testcase {
            name: "c",
            wasm: include_bytes!("../wasm/c_f32_counter.wasm"),
        },
        Testcase {
            name: "zig",
            wasm: include_bytes!("../wasm/zig_f32_counter.wasm"),
        },
    ];

    for testcase in testcases {
        let wasm = wasm_submemory::rewrite(testcase.wasm, SUBMEMORY_SIZE as i32)?;
        let mut vm = VM::new(&wasm)?;
        vm.set_memory_size(10 * SUBMEMORY_SIZE)?;
        for i in 1..=10 {
            for j in 0..10 {
                vm.call("set_base", &[Value::I32(j * SUBMEMORY_SIZE as i32)])?;
                let ret = vm.call("entry", &[])?;
                assert_eq!(*ret, [Value::F32(i as f32)], "{}", testcase.name);
            }
        }
    }

    Ok(())
}
