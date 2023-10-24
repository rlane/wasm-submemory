// TODO figure out if/when entry function runs
// TODO clean up integer types
// TODO support vector instructions
// TODO support memory instructions
//
// Memory layout:
// 1 page submemory bookkeeping ("headroom")
// K pages initial memory contents
// Submemory 0
// ...
// Submemory N
use walrus::{
    ir::*, ActiveDataLocation, FunctionBuilder, FunctionId, GlobalId, InitExpr, LocalFunction,
    ValType,
};

pub const WASM_PAGE_SIZE: u64 = 65536;
pub const HEADROOM_SIZE: u64 = WASM_PAGE_SIZE;

pub fn rewrite(wasm: &[u8], submemory_size: u64) -> anyhow::Result<Vec<u8>> {
    let mut module = walrus::Module::from_buffer(wasm)?;

    let num_mutable_globals = module.globals.iter().filter(|g| g.mutable).count();
    if num_mutable_globals > 1 {
        anyhow::bail!("wasm file has more than one mutable global");
    }

    let base_global = module
        .globals
        .add_local(ValType::I32, true, InitExpr::Value(Value::I32(0)));
    let index_global = module
        .globals
        .add_local(ValType::I32, true, InitExpr::Value(Value::I32(0)));
    let count_global = module
        .globals
        .add_local(ValType::I32, true, InitExpr::Value(Value::I32(0)));

    let memory_id;
    let initial_pages;
    if let Some(memory) = module.memories.iter_mut().next() {
        memory.maximum = None;
        let data_segment_ids = memory.data_segments.iter().cloned().collect::<Vec<_>>();
        for id in data_segment_ids {
            match &mut module.data.get_mut(id).kind {
                walrus::DataKind::Active(active) => match &mut active.location {
                    ActiveDataLocation::Absolute(ref mut offset) => *offset += HEADROOM_SIZE as u32,
                    ActiveDataLocation::Relative(_) => {
                        anyhow::bail!("unsupported relative data segment")
                    }
                },
                _ => {}
            }
        }
        initial_pages = memory.initial as u64;
        memory_id = memory.id();
        memory.initial += HEADROOM_SIZE as u32 / WASM_PAGE_SIZE as u32;
    } else {
        anyhow::bail!("wasm file has no memory");
    }

    let mut exempt_functions = Vec::new();

    // Create a select_submemory(index: i32) function.
    {
        let mut func = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
        let index = module.locals.add(ValType::I32);
        func.func_body()
            .local_get(index)
            .global_set(index_global)
            .local_get(index)
            .i32_const(submemory_size as i32)
            .binop(BinaryOp::I32Mul)
            .i32_const(HEADROOM_SIZE as i32 + initial_pages as i32 * WASM_PAGE_SIZE as i32)
            .binop(BinaryOp::I32Add)
            .global_set(base_global);
        let id = func.finish(vec![index], &mut module.funcs);
        module.exports.add("select_submemory", id);
        exempt_functions.push(id);
    }

    // Create an add_submemory() -> (index: i32, base_address: i32) function.
    {
        let mut func = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32, ValType::I32]);
        let base_address = module.locals.add(ValType::I32);
        func.func_body()
            // prev_pages = memory.grow(submemory_size / WASM_PAGE_SIZE)
            .i32_const(submemory_size as i32 / WASM_PAGE_SIZE as i32)
            .memory_grow(memory_id)
            // base_address = prev_pages * WASM_PAGE_SIZE
            .i32_const(WASM_PAGE_SIZE as i32)
            .binop(BinaryOp::I32Mul)
            .local_tee(base_address)
            // memory.copy(base_address, HEADROOM_SIZE, initial_pages * WASM_PAGE_SIZE)
            .i32_const(HEADROOM_SIZE as i32)
            .i32_const(initial_pages as i32 * WASM_PAGE_SIZE as i32)
            .memory_copy(memory_id, memory_id)
            // allocated_pages[count] = initial_pages
            .global_get(count_global)
            .i32_const(4)
            .binop(BinaryOp::I32Mul)
            .i32_const(initial_pages as i32)
            .store(
                memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 4,
                    offset: 0,
                },
            )
            // return (count++, base_address)
            .global_get(count_global)
            .local_get(base_address)
            .global_get(count_global)
            .i32_const(1)
            .binop(BinaryOp::I32Add)
            .global_set(count_global);
        let id = func.finish(vec![], &mut module.funcs);
        module.exports.add("add_submemory", id);
        exempt_functions.push(id);
    }

    // Create a fake_memory_grow(i32) -> i32 function.
    // TODO return -1 if the submemory is full
    let fake_memory_grow = {
        let mut func = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
        let delta_pages = module.locals.add(ValType::I32);
        let addr = module.locals.add(ValType::I32);
        let prev_pages = module.locals.add(ValType::I32);
        func.func_body()
            // addr = &allocated_pages[index]
            .global_get(index_global)
            .i32_const(4)
            .binop(BinaryOp::I32Mul)
            .local_tee(addr)
            // prev_pages = *addr
            .load(
                memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 4,
                    offset: 0,
                },
            )
            .local_set(prev_pages)
            // allocated_pages[index] += delta_pages
            .local_get(addr)
            .local_get(prev_pages)
            .local_get(delta_pages)
            .binop(BinaryOp::I32Add)
            .store(
                memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 4,
                    offset: 0,
                },
            )
            // return prev_pages
            .local_get(prev_pages);
        let id = func.finish(vec![delta_pages], &mut module.funcs);
        exempt_functions.push(id);
        id
    };

    // Create a fake_memory_size() -> i32 function.
    let fake_memory_size = {
        let mut func = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);
        func.func_body()
            .global_get(index_global)
            .i32_const(4)
            .binop(BinaryOp::I32Mul)
            .load(
                memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 4,
                    offset: 0,
                },
            );
        let id = func.finish(vec![], &mut module.funcs);
        exempt_functions.push(id);
        id
    };

    let saved_values = SavedValues::new(&mut module);
    let context = Context {
        base_global,
        submemory_size,
        saved_values,
        fake_memory_grow,
        fake_memory_size,
    };
    for (id, func) in module.funcs.iter_local_mut() {
        if exempt_functions.contains(&id) {
            continue;
        }
        rewrite_function(func, &context)?;
    }

    Ok(module.emit_wasm())
}

struct Context {
    base_global: GlobalId,
    submemory_size: u64,
    saved_values: SavedValues,
    fake_memory_grow: FunctionId,
    fake_memory_size: FunctionId,
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
                new_instrs.push((
                    Instr::Call(Call {
                        func: context.fake_memory_size,
                    }),
                    *instr_loc_id,
                ));
            }
            Instr::MemoryGrow(_) => {
                new_instrs.push((
                    Instr::Call(Call {
                        func: context.fake_memory_grow,
                    }),
                    *instr_loc_id,
                ));
            }
            Instr::MemoryInit(_)
            | Instr::MemoryCopy(_)
            | Instr::MemoryFill(_)
            | Instr::LoadSimd(_)
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
