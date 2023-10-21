use testresult::TestResult;
use wasmer::{imports, Instance, Module, Store, Value};

#[test]
fn i32_works() -> TestResult {
    let submemory_size: i32 = 1 << 20;
    let wasm = include_bytes!("../wasm/test1.wasm");
    let wasm = wasm_submemory::rewrite(wasm, submemory_size)?;
    let mut store = Store::default();
    let import_object = imports! {};
    let module = Module::new(&store, &wasm)?;
    let instance = Instance::new(&mut store, &module, &import_object)?;
    let memory = instance.exports.get_memory("memory")?.clone();
    memory.grow(&mut store, (10 * submemory_size / 65536) as u32)?;
    let entry = instance.exports.get_function("entry")?.clone();
    let set_base = instance.exports.get_function("set_base")?.clone();
    for i in 1..=10 {
        for j in 0..10 {
            set_base.call(&mut store, &[Value::I32(j * submemory_size)])?;
            assert_eq!(returned_int(&entry.call(&mut store, &[])?)?, i);
        }
    }
    Ok(())
}

fn returned_int(result: &[Value]) -> anyhow::Result<i32> {
    match result.len() {
        0 => anyhow::bail!("no return value"),
        1 => match result[0] {
            Value::I32(i) => Ok(i),
            _ => anyhow::bail!("return value is not an i32"),
        },
        _ => anyhow::bail!("too many return values"),
    }
}
