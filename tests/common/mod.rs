use wasmer::{imports, Instance, Module, Store, Value};

pub const WASM_PAGE_SIZE: u64 = 65536;
pub const SUBMEMORY_SIZE: u64 = 1 << 20;

pub fn parse_wat(wat: &str) -> anyhow::Result<Vec<u8>> {
    Ok(wasmer::wat2wasm(wat.as_bytes())?.to_vec())
}

pub struct VM {
    store: Store,
    instance: Instance,
    memory: wasmer::Memory,
}

impl VM {
    pub fn new(wasm: &[u8]) -> anyhow::Result<Self> {
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

    pub fn call(&mut self, func_name: &str, args: &[Value]) -> anyhow::Result<Box<[Value]>> {
        Ok(self
            .instance
            .exports
            .get_function(func_name)?
            .call(&mut self.store, args)?)
    }

    pub fn set_memory_size(&mut self, size: u64) -> anyhow::Result<()> {
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

pub fn returned_int(result: &[Value]) -> anyhow::Result<i32> {
    match result.len() {
        0 => anyhow::bail!("no return value"),
        1 => match result[0] {
            Value::I32(i) => Ok(i),
            _ => anyhow::bail!("return value is not an i32"),
        },
        _ => anyhow::bail!("too many return values"),
    }
}