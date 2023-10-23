mod common;

use crate::common::*;
use testresult::TestResult;
use wasmer::Value;

struct Testcase<'a> {
    name: &'a str,
    wasm: &'a [u8],
}

#[test]
fn allocation() -> TestResult {
    let testcases = &[
        Testcase {
            name: "rust",
            wasm: include_bytes!("../testdata/wasm/rust/allocation.wasm"),
        },
        Testcase {
            name: "zig",
            wasm: include_bytes!("../testdata/wasm/zig/allocation.wasm"),
        },
    ];

    for testcase in testcases {
        let wasm = wasm_submemory::rewrite(testcase.wasm, SUBMEMORY_SIZE)?;
        let mut vm = VM::new(&wasm)?;
        for i in 0..10 {
            assert_eq!(vm.add_submemory()?.0, i);
        }
        for i in 0..10 {
            vm.select_submemory(i)?;
            let ret = vm.call("entry", &[])?;
            assert_eq!(*ret, [Value::I32(42)], "{} {}", testcase.name, i);
        }
    }

    Ok(())
}
