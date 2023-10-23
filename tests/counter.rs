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
            wasm: include_bytes!("../testdata/wasm/rust/i32_counter.wasm"),
        },
        Testcase {
            name: "c",
            wasm: include_bytes!("../testdata/wasm/c/i32_counter.wasm"),
        },
        Testcase {
            name: "zig",
            wasm: include_bytes!("../testdata/wasm/zig/i32_counter.wasm"),
        },
    ];

    for testcase in testcases {
        let wasm = wasm_submemory::rewrite(testcase.wasm, SUBMEMORY_SIZE)?;
        let mut vm = VM::new(&wasm)?;
        for i in 0..10 {
            assert_eq!(vm.add_submemory()?, i);
        }
        for i in 1..=10 {
            for j in 0..10 {
                vm.select_submemory(j)?;
                let ret = vm.call("entry", &[])?;
                assert_eq!(*ret, [Value::I32(i as i32)], "{} {}", testcase.name, i);
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
            wasm: include_bytes!("../testdata/wasm/rust/f32_counter.wasm"),
        },
        Testcase {
            name: "c",
            wasm: include_bytes!("../testdata/wasm/c/f32_counter.wasm"),
        },
        Testcase {
            name: "zig",
            wasm: include_bytes!("../testdata/wasm/zig/f32_counter.wasm"),
        },
    ];

    for testcase in testcases {
        let wasm = wasm_submemory::rewrite(testcase.wasm, SUBMEMORY_SIZE)?;
        let mut vm = VM::new(&wasm)?;
        for i in 0..10 {
            assert_eq!(vm.add_submemory()?, i);
        }
        for i in 1..=10 {
            for j in 0..10 {
                vm.select_submemory(j)?;
                let ret = vm.call("entry", &[])?;
                assert_eq!(*ret, [Value::F32(i as f32)], "{}", testcase.name);
            }
        }
    }

    Ok(())
}

#[test]
fn i64_counter() -> TestResult {
    let wat_wasm = parse_wat(
        r#"
(module
  (type (;0;) (func (result i64)))
  (func $entry (type 0) (result i64)
    (local i64)
    i32.const 0
    i32.const 0
    i64.load offset=64
    i64.const 1
    i64.add
    local.tee 0
    i64.store offset=64
    local.get 0)
  (memory (;0;) 1)
  (export "memory" (memory 0))
  (export "entry" (func $entry)))
            "#,
    )?;

    let testcases = &[Testcase {
        name: "wat",
        wasm: &wat_wasm,
    }];

    for testcase in testcases {
        let wasm = wasm_submemory::rewrite(testcase.wasm, SUBMEMORY_SIZE)?;
        let mut vm = VM::new(&wasm)?;
        for i in 0..10 {
            assert_eq!(vm.add_submemory()?, i);
        }
        for i in 1..=10 {
            for j in 0..10 {
                vm.select_submemory(j)?;
                let ret = vm.call("entry", &[])?;
                assert_eq!(*ret, [Value::I64(i as i64)], "{}", testcase.name);
            }
        }
    }

    Ok(())
}

#[test]
fn i64_8_counter() -> TestResult {
    let wat_wasm = parse_wat(
        r#"
(module
  (type (;0;) (func (result i64)))
  (func $entry (type 0) (result i64)
    (local i64)
    i32.const 0
    i32.const 0
    i64.load8_u offset=64
    i64.const 1
    i64.add
    local.tee 0
    i64.store8 offset=64
    local.get 0)
  (memory (;0;) 1)
  (export "memory" (memory 0))
  (export "entry" (func $entry)))
            "#,
    )?;

    let testcases = &[Testcase {
        name: "wat",
        wasm: &wat_wasm,
    }];

    for testcase in testcases {
        let wasm = wasm_submemory::rewrite(testcase.wasm, SUBMEMORY_SIZE)?;
        let mut vm = VM::new(&wasm)?;
        for i in 0..10 {
            assert_eq!(vm.add_submemory()?, i);
        }
        for i in 1..=10 {
            for j in 0..10 {
                vm.select_submemory(j)?;
                let ret = vm.call("entry", &[])?;
                assert_eq!(*ret, [Value::I64(i as i64)], "{}", testcase.name);
            }
        }
    }

    Ok(())
}

#[test]
fn f64_counter() -> TestResult {
    let wat_wasm = parse_wat(
        r#"
(module
  (type (;0;) (func (result f64)))
  (func $entry (type 0) (result f64)
    (local f64)
    i32.const 0
    i32.const 0
    f64.load offset=64
    f64.const 1
    f64.add
    local.tee 0
    f64.store offset=64
    local.get 0)
  (memory (;0;) 1)
  (export "memory" (memory 0))
  (export "entry" (func $entry)))
            "#,
    )?;

    let testcases = &[Testcase {
        name: "wat",
        wasm: &wat_wasm,
    }];

    for testcase in testcases {
        let wasm = wasm_submemory::rewrite(testcase.wasm, SUBMEMORY_SIZE)?;
        let mut vm = VM::new(&wasm)?;
        for i in 0..10 {
            assert_eq!(vm.add_submemory()?, i);
        }
        for i in 1..=10 {
            for j in 0..10 {
                vm.select_submemory(j)?;
                let ret = vm.call("entry", &[])?;
                assert_eq!(*ret, [Value::F64(i as f64)], "{}", testcase.name);
            }
        }
    }

    Ok(())
}
