use std::fs;
use std::process::Command;
use quicli::prelude::*;
use structopt::StructOpt;
use toml::{self};
mod definition;

#[derive(Debug, StructOpt)]
struct Cli {
    //#[structopt(long="output", short="o", default_value="myContract")]
    //output: String,
    /// The file to read
    file: String
}

fn main() {
    let args = Cli::from_args();
    let config = fs::read_to_string(&args.file).expect("Unable to read file");
    let mut source_val: definition::ContractDefinitions = toml::from_str(config.as_str()).unwrap();
    //assert_eq!(source_val.name, "myContract");
    println!("{}", source_val.name);
    source_val.process_types();
    let filename = format!("{}.rs", source_val.name);
    // do stuff with the config, serialize and print out contract templates
    let output = definition::fill_template(&source_val);
    fs::write(&filename, output).expect("Unable to write to file");
    Command::new("rustfmt").arg(&filename).output().expect("Failed to execute command");
}