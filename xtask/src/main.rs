#[macro_use]
extern crate html5ever;

use {clap::StructOpt, std::path::PathBuf};

mod docgen;

#[derive(clap::Parser)]
#[clap(about, long_about = None)]
enum Cli {
    Docgen { fmod_path: PathBuf },
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let Cli::Docgen { fmod_path } = Cli::parse();
    docgen::main(&fmod_path)
}
