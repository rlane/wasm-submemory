#![allow(dead_code)]

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
    initial_contents: Vec<u8>,
}

impl VM {
    pub fn new(wasm: &[u8]) -> anyhow::Result<Self> {
        let mut store = Store::default();
        let import_object = imports! {};
        let module = Module::new(&store, wasm)?;
        let instance = Instance::new(&mut store, &module, &import_object)?;
        let memory = instance.exports.get_memory("memory")?.clone();
        let initial_contents = memory.view(&mut store).copy_to_vec()?;
        Ok(VM {
            store,
            instance,
            memory,
            initial_contents,
        })
    }

    pub fn call(&mut self, func_name: &str, args: &[Value]) -> anyhow::Result<Box<[Value]>> {
        Ok(self
            .instance
            .exports
            .get_function(func_name)?
            .call(&mut self.store, args)?)
    }
}
