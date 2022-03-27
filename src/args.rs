use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[clap(version)]
pub struct Args {
    /// Config file path
    #[clap(short, long, default_value = "config.toml")]
    pub config: String,
}

pub struct RetArgs {
    pub config: PathBuf,
}

pub fn get() -> RetArgs {
    let mut args = Args::parse();
    let mut ret_args = RetArgs {
        config: Path::new(&args.config).to_path_buf(),
    };

    if !args.config.ends_with(".toml") {
        args.config = format!("{}.toml", args.config);
    }

    if !ret_args.config.exists() {
        let with_toml = Path::new(&args.config).with_extension("toml");
        if with_toml.exists() {
            ret_args.config = with_toml;
        } else {
            println!(
                "Config file not found: {}, {}",
                &ret_args.config.to_str().unwrap(),
                &with_toml.to_str().unwrap()
            );
            std::process::exit(1);
        }
    }

    println!("Using config file: {}", &ret_args.config.to_str().unwrap());

    ret_args
}
