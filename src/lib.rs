use walrus::{ir::*, FunctionBuilder, GlobalId, InitExpr, LocalFunction, ValType};

// TODO need to support more types
struct SavedValues {
    val_i32: LocalId,
}

pub fn rewrite(wasm: &[u8], limit: i32) -> anyhow::Result<Vec<u8>> {
    let mut module = walrus::Module::from_buffer(wasm)?;
    let base_global = module
        .globals
        .add_local(ValType::I32, true, InitExpr::Value(Value::I32(0)));

    // Create a set_base() function.
    {
        let mut func = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
        let base = module.locals.add(ValType::I32);
        func.func_body().local_get(base).global_set(base_global);
        let set_base = func.finish(vec![base], &mut module.funcs);
        module.exports.add("set_base", set_base);
    }

    let saved_values = SavedValues {
        val_i32: module.locals.add(ValType::I32),
    };

    for (_, func) in module.funcs.iter_local_mut() {
        rewrite_function(func, base_global, limit, &saved_values);
    }

    Ok(module.emit_wasm())
}

fn rewrite_function(
    func: &mut LocalFunction,
    base_global: GlobalId,
    limit: i32,
    saved_values: &SavedValues,
) {
    let block_ids: Vec<_> = func.blocks().map(|(block_id, _block)| block_id).collect();
    for block_id in block_ids {
        rewrite_block(func, block_id, base_global, limit, saved_values);
    }
}

fn rewrite_block(
    func: &mut LocalFunction,
    block_id: InstrSeqId,
    base_global: GlobalId,
    limit: i32,
    saved_values: &SavedValues,
) {
    let block = func.block_mut(block_id);
    let block_instrs = &mut block.instrs;
    let mask = limit - 1;

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
                            global: base_global,
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
                let bounds_checked_instrs = &[
                    (
                        Instr::LocalSet(LocalSet {
                            local: saved_values.val_i32,
                        }),
                        InstrLocId::default(),
                    ),
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
                            global: base_global,
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
                        Instr::LocalGet(LocalGet {
                            local: saved_values.val_i32,
                        }),
                        InstrLocId::default(),
                    ),
                    (Instr::Store(new_store), *instr_loc_id),
                ];
                new_instrs.extend(bounds_checked_instrs.iter().cloned());
            }
            _ => {
                new_instrs.push((instr.clone(), *instr_loc_id));
            }
        }
    }

    block.instrs = new_instrs;
}