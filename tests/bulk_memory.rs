mod common;

use crate::common::*;
use testresult::TestResult;
use walrus::ir::{LoadKind, StoreKind};
use wasmer::Value;

#[test]
fn memory_init() -> TestResult {
    let mut module = walrus::Module::default();
    let memory = module.memories.add_local(false, false, 1, None, None);
    let data = module.data.add(walrus::DataKind::Passive, vec![42; 4]);

    let mut builder = walrus::FunctionBuilder::new(&mut module.types, &[], &[walrus::ValType::I32]);
    builder
        .func_body()
        .i32_const(64) // dest
        .i32_const(0) // source
        .i32_const(4) // size
        .memory_init(memory, data)
        .i32_const(64)
        .load(
            memory,
            LoadKind::I32 { atomic: false },
            walrus::ir::MemArg {
                align: 4,
                offset: 0,
            },
        ); // Result i32

    let func_id = builder.finish(vec![], &mut module.funcs);
    module.exports.add("memory", memory);
    module.exports.add("entry", func_id);

    let wasm = module.emit_wasm();

    let rewritten_wasm = wasm_submemory::rewrite(&wasm, SUBMEMORY_SIZE)?;
    let mut vm = VM::new(&rewritten_wasm)?;

    let base_address = WASM_PAGE_SIZE * 2;
    assert_eq!(vm.add_submemory()?, (0, base_address));

    vm.select_submemory(0)?;
    let ret = vm.call("entry", &[])?;
    // bytes are 42 (0x2A). 4 bytes = 0x2A2A2A2A = 707406378
    assert_eq!(*ret, [Value::I32(707406378)]);

    Ok(())
}

#[test]
fn memory_copy() -> TestResult {
    let mut module = walrus::Module::default();
    let memory = module.memories.add_local(false, false, 1, None, None);

    let mut builder = walrus::FunctionBuilder::new(&mut module.types, &[], &[walrus::ValType::I32]);
    builder
        .func_body()
        .i32_const(0)
        .i32_const(42)
        .store(
            memory,
            StoreKind::I32 { atomic: false },
            walrus::ir::MemArg {
                align: 4,
                offset: 0,
            },
        )
        .i32_const(64) // dest
        .i32_const(0) // source
        .i32_const(4) // size
        .memory_copy(memory, memory)
        .i32_const(64)
        .load(
            memory,
            LoadKind::I32 { atomic: false },
            walrus::ir::MemArg {
                align: 4,
                offset: 0,
            },
        );

    let func_id = builder.finish(vec![], &mut module.funcs);
    module.exports.add("memory", memory);
    module.exports.add("entry", func_id);

    let wasm = module.emit_wasm();
    let rewritten_wasm = wasm_submemory::rewrite(&wasm, SUBMEMORY_SIZE)?;
    let mut vm = VM::new(&rewritten_wasm)?;

    let base_address = WASM_PAGE_SIZE * 2;
    assert_eq!(vm.add_submemory()?, (0, base_address));

    vm.select_submemory(0)?;
    let ret = vm.call("entry", &[])?;
    assert_eq!(*ret, [Value::I32(42)]);

    Ok(())
}

#[test]
fn memory_fill() -> TestResult {
    let mut module = walrus::Module::default();
    let memory = module.memories.add_local(false, false, 1, None, None);

    let mut builder = walrus::FunctionBuilder::new(&mut module.types, &[], &[walrus::ValType::I32]);
    builder
        .func_body()
        .i32_const(64) // dest
        .i32_const(42) // val
        .i32_const(1) // size
        .memory_fill(memory)
        .i32_const(64)
        .load(
            memory,
            LoadKind::I32_8 {
                kind: walrus::ir::ExtendedLoad::ZeroExtend,
            },
            walrus::ir::MemArg {
                align: 0,
                offset: 0,
            },
        );

    let func_id = builder.finish(vec![], &mut module.funcs);
    module.exports.add("memory", memory);
    module.exports.add("entry", func_id);

    let wasm = module.emit_wasm();
    let rewritten_wasm = wasm_submemory::rewrite(&wasm, SUBMEMORY_SIZE)?;
    let mut vm = VM::new(&rewritten_wasm)?;

    let base_address = WASM_PAGE_SIZE * 2;
    assert_eq!(vm.add_submemory()?, (0, base_address));

    vm.select_submemory(0)?;
    let ret = vm.call("entry", &[])?;
    assert_eq!(*ret, [Value::I32(42)]);

    Ok(())
}
