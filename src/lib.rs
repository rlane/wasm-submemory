use walrus::{
    ir::*, ActiveDataLocation, FunctionBuilder, GlobalId, InitExpr, LocalFunction, ValType,
};

pub const WASM_PAGE_SIZE: u64 = 65536;
pub const HEADROOM: u64 = 1 << 20;

pub fn rewrite(wasm: &[u8], submemory_size: u64) -> anyhow::Result<Vec<u8>> {
    let mut module = walrus::Module::from_buffer(wasm)?;

    let num_mutable_globals = module.globals.iter().filter(|g| g.mutable).count();
    if num_mutable_globals > 1 {
        anyhow::bail!("wasm file has more than one mutable global");
    }

    let base_global = module
        .globals
        .add_local(ValType::I32, true, InitExpr::Value(Value::I32(0)));

    {
        let memory = module.memories.iter_mut().next().unwrap();
        memory.maximum = None;
        let data_segment_ids = memory.data_segments.iter().cloned().collect::<Vec<_>>();
        for id in data_segment_ids {
            match &mut module.data.get_mut(id).kind {
                walrus::DataKind::Active(active) => match &mut active.location {
                    ActiveDataLocation::Absolute(ref mut offset) => *offset += HEADROOM as u32,
                    ActiveDataLocation::Relative(_) => todo!("relative data segment"),
                },
                _ => {}
            }
        }
        memory.initial += HEADROOM as u32 / WASM_PAGE_SIZE as u32;
    };

    // Create a set_base() function.
    {
        let mut func = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
        let base = module.locals.add(ValType::I32);
        func.func_body()
            .local_get(base)
            .i32_const(HEADROOM as i32)
            .binop(BinaryOp::I32Add)
            .global_set(base_global);
        let set_base = func.finish(vec![base], &mut module.funcs);
        module.exports.add("set_base", set_base);
    }

    let saved_values = SavedValues::new(&mut module);
    let context = Context {
        base_global,
        submemory_size,
        saved_values,
    };
    for (_, func) in module.funcs.iter_local_mut() {
        rewrite_function(func, &context)?;
    }

    Ok(module.emit_wasm())
}

struct Context {
    base_global: GlobalId,
    submemory_size: u64,
    saved_values: SavedValues,
}

fn rewrite_function(func: &mut LocalFunction, context: &Context) -> anyhow::Result<()> {
    let block_ids: Vec<_> = func.blocks().map(|(block_id, _block)| block_id).collect();
    for block_id in block_ids {
        rewrite_block(func, block_id, context)?;
    }
    Ok(())
}

fn rewrite_block(
    func: &mut LocalFunction,
    block_id: InstrSeqId,
    context: &Context,
) -> anyhow::Result<()> {
    let block = func.block_mut(block_id);
    let block_instrs = &mut block.instrs;
    let mask = context.submemory_size - 1;

    // TODO need to support more memory instructions
    let mut new_instrs: Vec<(Instr, InstrLocId)> = vec![];
    for (instr, instr_loc_id) in block_instrs.iter() {
        match instr {
            Instr::Load(load) => {
                use walrus::ir::Value::*;
                let mut new_load = load.clone();
                new_load.arg.offset = 0;
                let bounds_checked_instrs = &[
                    (
                        Instr::Const(Const {
                            value: I32(load.arg.offset as i32),
                        }),
                        InstrLocId::default(),
                    ),
                    (
                        Instr::Binop(Binop {
                            op: BinaryOp::I32Add,
                        }),
                        InstrLocId::default(),
                    ),
                    (
                        Instr::Const(Const {
                            value: I32(mask as i32),
                        }),
                        InstrLocId::default(),
                    ),
                    (
                        Instr::Binop(Binop {
                            op: BinaryOp::I32And,
                        }),
                        InstrLocId::default(),
                    ),
                    (
                        Instr::GlobalGet(GlobalGet {
                            global: context.base_global,
                        }),
                        InstrLocId::default(),
                    ),
                    (
                        Instr::Binop(Binop {
                            op: BinaryOp::I32Add,
                        }),
                        InstrLocId::default(),
                    ),
                    (Instr::Load(new_load), *instr_loc_id),
                ];
                new_instrs.extend(bounds_checked_instrs.iter().cloned());
            }
            Instr::Store(store) => {
                use walrus::ir::Value::*;
                let mut new_store = store.clone();
                new_store.arg.offset = 0;
                let local = context.saved_values.get(store.kind)?;
                let bounds_checked_instrs = &[
                    (Instr::LocalSet(LocalSet { local }), InstrLocId::default()),
                    (
                        Instr::Const(Const {
                            value: I32(store.arg.offset as i32),
                        }),
                        InstrLocId::default(),
                    ),
                    (
                        Instr::Binop(Binop {
                            op: BinaryOp::I32Add,
                        }),
                        InstrLocId::default(),
                    ),
                    (
                        Instr::Const(Const {
                            value: I32(mask as i32),
                        }),
                        InstrLocId::default(),
                    ),
                    (
                        Instr::Binop(Binop {
                            op: BinaryOp::I32And,
                        }),
                        InstrLocId::default(),
                    ),
                    (
                        Instr::GlobalGet(GlobalGet {
                            global: context.base_global,
                        }),
                        InstrLocId::default(),
                    ),
                    (
                        Instr::Binop(Binop {
                            op: BinaryOp::I32Add,
                        }),
                        InstrLocId::default(),
                    ),
                    (Instr::LocalGet(LocalGet { local }), InstrLocId::default()),
                    (Instr::Store(new_store), *instr_loc_id),
                ];
                new_instrs.extend(bounds_checked_instrs.iter().cloned());
            }
            Instr::MemorySize(_) => {
                todo!("memory size");
            }
            Instr::MemoryGrow(_) => {
                todo!("memory grow");
            }
            Instr::MemoryInit(_) => {
                todo!("memory init");
            }
            Instr::MemoryCopy(_) => {
                todo!("memory copy");
            }
            Instr::MemoryFill(_) => {
                todo!("memory fill");
            }
            Instr::LoadSimd(_)
            | Instr::Cmpxchg(_)
            | Instr::AtomicRmw(_)
            | Instr::AtomicWait(_)
            | Instr::AtomicNotify(_) => {
                anyhow::bail!("unsupported instruction: {:?}", instr);
            }
            _ => {
                new_instrs.push((instr.clone(), *instr_loc_id));
            }
        }
    }

    block.instrs = new_instrs;
    Ok(())
}

// TODO need to support more types
struct SavedValues {
    val_i32: LocalId,
    val_f32: LocalId,
    val_i64: LocalId,
    val_f64: LocalId,
}

impl SavedValues {
    fn new(module: &mut walrus::Module) -> Self {
        Self {
            val_i32: module.locals.add(ValType::I32),
            val_f32: module.locals.add(ValType::F32),
            val_i64: module.locals.add(ValType::I64),
            val_f64: module.locals.add(ValType::F64),
        }
    }

    fn get(&self, store_kind: StoreKind) -> anyhow::Result<LocalId> {
        Ok(match store_kind {
            walrus::ir::StoreKind::I32 { .. } => self.val_i32,
            walrus::ir::StoreKind::I32_8 { .. } => self.val_i32,
            walrus::ir::StoreKind::I32_16 { .. } => self.val_i32,
            walrus::ir::StoreKind::I64 { .. } => self.val_i64,
            walrus::ir::StoreKind::I64_8 { .. } => self.val_i64,
            walrus::ir::StoreKind::I64_16 { .. } => self.val_i64,
            walrus::ir::StoreKind::I64_32 { .. } => self.val_i64,
            walrus::ir::StoreKind::F32 => self.val_f32,
            walrus::ir::StoreKind::F64 { .. } => self.val_f64,
            _ => {
                anyhow::bail!("unsupported store kind {:?}", store_kind);
            }
        })
    }
}
