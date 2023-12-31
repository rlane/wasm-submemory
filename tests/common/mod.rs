#![allow(dead_code)]

use wasmer::{imports, Instance, Module, Store, Value};

pub const WASM_PAGE_SIZE: u32 = wasm_submemory::WASM_PAGE_SIZE;
pub const SUBMEMORY_SIZE: u32 = 1 << 20;

pub fn parse_wat(wat: &str) -> anyhow::Result<Vec<u8>> {
    Ok(wasmer::wat2wasm(wat.as_bytes())?.to_vec())
}

pub struct VM {
    pub store: Store,
    pub instance: Instance,
    pub memory: wasmer::Memory,
    pub initial_contents: Vec<u8>,
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

    pub fn add_submemory(&mut self) -> anyhow::Result<(u32, u32)> {
        match *self.call("add_submemory", &[])? {
            [Value::I32(index), Value::I32(base_address)] => {
                Ok((index as u32, base_address as u32))
            }
            _ => Err(anyhow::anyhow!("unexpected result from add_submemory")),
        }
    }

    pub fn select_submemory(&mut self, index: u32) -> anyhow::Result<()> {
        self.call("select_submemory", &[Value::I32(index as i32)])?;
        Ok(())
    }

    pub fn reset_submemory(&mut self, index: u32) -> anyhow::Result<()> {
        self.call("reset_submemory", &[Value::I32(index as i32)])?;
        Ok(())
    }
}
