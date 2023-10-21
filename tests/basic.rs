use testresult::TestResult;
use wasmer::{imports, Instance, Module, Store, Value};

const WASM_PAGE_SIZE: u64 = 65536;
const SUBMEMORY_SIZE: u64 = 1 << 20;

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

fn parse_wat(wat: &str) -> anyhow::Result<Vec<u8>> {
    Ok(wasmer::wat2wasm(wat.as_bytes())?.to_vec())
}

struct VM {
    store: Store,
    instance: Instance,
    memory: wasmer::Memory,
}

impl VM {
    fn new(wasm: &[u8]) -> anyhow::Result<Self> {
        let mut store = Store::default();
        let import_object = imports! {};
        let module = Module::new(&store, wasm)?;
        let instance = Instance::new(&mut store, &module, &import_object)?;
        let memory = instance.exports.get_memory("memory")?.clone();
        Ok(VM {
            store,
            instance,
            memory,
        })
    }

    fn call(&mut self, func_name: &str, args: &[Value]) -> anyhow::Result<Box<[Value]>> {
        Ok(self
            .instance
            .exports
            .get_function(func_name)?
            .call(&mut self.store, args)?)
    }

    fn set_memory_size(&mut self, size: u64) -> anyhow::Result<()> {
        if size % WASM_PAGE_SIZE != 0 {
            anyhow::bail!("memory size must be a multiple of {}", WASM_PAGE_SIZE);
        }
        let current_size = self.memory.view(&mut self.store).data_size();
        if size < current_size {
            anyhow::bail!("cannot shrink memory from {} to {}", current_size, size);
        }
        let delta_pages = (size - current_size) / WASM_PAGE_SIZE;
        self.memory.grow(&mut self.store, delta_pages as u32)?;
        Ok(())
    }
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
