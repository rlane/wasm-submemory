mod common;

use crate::common::*;
use testresult::TestResult;
use wasmer::Value;

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
        let wasm = wasm_submemory::rewrite(testcase.wasm, SUBMEMORY_SIZE as i32)?;
        let mut vm = VM::new(&wasm)?;
        vm.set_memory_size(10 * SUBMEMORY_SIZE)?;
        for i in 0..10 {
            vm.init_submemory(i)?;
            vm.call("set_base", &[Value::I32(i * SUBMEMORY_SIZE as i32)])?;
            let ret = vm.call("entry", &[])?;
            assert_eq!(*ret, [Value::I32(42)], "{} {}", testcase.name, i);
        }
    }

    Ok(())
}
