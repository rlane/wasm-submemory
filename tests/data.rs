mod common;

use crate::common::*;
use testresult::TestResult;
use wasmer::{Value, WasmPtr};

struct Testcase<'a> {
    name: &'a str,
    wasm: &'a [u8],
}

#[test]
fn data() -> TestResult {
    let wat_wasm = parse_wat(
        r#"
(module
  (type (;0;) (func (result i32)))
  (func $entry (type 0) (result i32)
    i32.const 0
    i32.load offset=64)
  (memory (;0;) 1)
  (data $.data (i32.const 64) "*\00\00\00")
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
            wasm: include_bytes!("../testdata/wasm/rust/data.wasm"),
        },
        Testcase {
            name: "c",
            wasm: include_bytes!("../testdata/wasm/c/data.wasm"),
        },
        Testcase {
            name: "zig",
            wasm: include_bytes!("../testdata/wasm/zig/data.wasm"),
        },
    ];

    for testcase in testcases {
        let wasm = wasm_submemory::rewrite(testcase.wasm, SUBMEMORY_SIZE)?;
        let mut vm = VM::new(&wasm)?;
        for i in 0..10 {
            assert_eq!(vm.add_submemory()?.0, i);
        }
        for i in 0..10 {
            vm.call("select_submemory", &[Value::I32(i)])?;
            let ret = vm.call("entry", &[])?;
            assert_eq!(*ret, [Value::I32(42)], "{} {}", testcase.name, i);
        }
    }

    Ok(())
}

#[test]
fn base_address() -> TestResult {
    let wat_wasm = parse_wat(
        r#"
(module
  (type (;0;) (func (result i32)))
  (func $entry (type 0) (result i32)
    i32.const 0
    i32.load offset=64)
  (memory (;0;) 1)
  (data $.data (i32.const 64) "*\00\00\00")
  (export "memory" (memory 0))
  (export "entry" (func $entry)))
            "#,
    )?;

    let testcases = &[Testcase {
        name: "wat",
        wasm: &wat_wasm,
    }];

    let offset = 64;

    for testcase in testcases {
        let wasm = wasm_submemory::rewrite(testcase.wasm, SUBMEMORY_SIZE)?;
        let mut vm = VM::new(&wasm)?;

        let base_address = |i| WASM_PAGE_SIZE * 2 + SUBMEMORY_SIZE * i;
        for i in 0..10 {
            assert_eq!(vm.add_submemory()?, (i, base_address(i)));
        }
        for i in 0..10 {
            vm.select_submemory(i)?;
            let ptr = WasmPtr::<i32>::new((base_address(i) + offset) as u32);
            for j in 0..10 {
                ptr.write(&vm.memory.view(&mut vm.store), 42 + j)?;
                let ret = vm.call("entry", &[])?;
                assert_eq!(*ret, [Value::I32(42 + j)], "{} {}", testcase.name, i);
            }
        }
    }

    Ok(())
}

#[test]
fn reset() -> TestResult {
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

    let wasm = wasm_submemory::rewrite(&wat_wasm, SUBMEMORY_SIZE)?;
    let mut vm = VM::new(&wasm)?;

    let base_address = WASM_PAGE_SIZE * 2;
    assert_eq!(vm.add_submemory()?, (0, base_address));

    for i in 0..2 {
        vm.select_submemory(0)?;
        let ret = vm.call("entry", &[])?;
        assert_eq!(*ret, [Value::I32(1)], "{i}");
        let ret = vm.call("entry", &[])?;
        assert_eq!(*ret, [Value::I32(2)], "{i}");
        vm.reset_submemory(0)?;
    }

    Ok(())
}
