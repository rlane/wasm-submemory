use clap::Parser;

#[derive(Parser, Debug)]
#[clap()]
struct Arguments {
    filename: String,
    #[clap(short, long)]
    output_filename: String,
}

fn main() -> anyhow::Result<()> {
    let submemory_size: u32 = 1 << 20;
    env_logger::init();
    let args = Arguments::parse();
    let wasm = std::fs::read(args.filename)?;
    let wasm = wasm_submemory::rewrite(&wasm, submemory_size)?;
    std::fs::write(args.output_filename, wasm)?;
    Ok(())
}
