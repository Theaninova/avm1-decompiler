pub mod ast;
pub mod decompiler;

use crate::ast::action::Action;
use crate::decompiler::{decompile, VmData};
use clap::{arg, Parser, Subcommand};
use dprint_plugin_typescript::configuration::{Configuration, ConfigurationBuilder};
use dprint_plugin_typescript::format_text;
use std::fs;
use std::path::Path;
use swf::avm1::read::Reader;
use swf::extensions::ReadSwfExt;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Decompile .abc files
    Decompile {
        /// Reject when occurring parse errors
        #[arg(short, long, default_value_t = false)]
        strict: bool,

        /// Specify a path to the constant pool file.
        /// By default it will look for a .json file
        /// with the same name
        #[arg(short, long)]
        pool: Option<std::path::PathBuf>,

        /// Output path of the decompiled result
        /// If not specified it will write next to the input with a .as extension
        #[arg(short, long)]
        out: Option<std::path::PathBuf>,

        /// Path to the .abc file
        path: std::path::PathBuf,
    },
}

fn main() {
    match Args::parse().cmd {
        Commands::Decompile {
            strict,
            path,
            out,
            pool,
        } => {
            let format_config = ConfigurationBuilder::new().build();

            let pool: Vec<String> = serde_json::from_str(
                fs::read_to_string(if let Some(pool_path) = pool {
                    pool_path
                } else {
                    path.with_extension("json")
                })
                .expect("Missing constant pool file")
                .as_str(),
            )
            .expect("Invalid pool file");
            let out_path = if let Some(out_path) = out {
                out_path
            } else {
                path.with_extension("as")
            };

            let data = fs::read(path).expect("Invalid file");
            let mut reader = Reader::new(&data, 1);

            let num_actions = reader.read_u32().unwrap();
            println!("{} actions", num_actions);

            for i in 0..num_actions {
                let action_size = reader.read_u16().unwrap();
                reader.read_u16().unwrap();
                let action = reader.read_slice(action_size as usize).unwrap();
                let action_body = decompile(VmData {
                    bytecode: action,
                    constant_pool: &pool,
                    strict,
                    registers: Vec::new(),
                })
                .expect("Decompile failed");

                let decompiled_action = Action {
                    id: i,
                    statements: action_body,
                };

                let emitted_code = decompiled_action.to_string();
                let formatted =
                    match format_text(Path::new("action.js"), &emitted_code, &format_config) {
                        Ok(code) => code,
                        Err(error) => {
                            eprintln!("Formatting failed: {}", error);
                            None
                        }
                    }
                    .unwrap_or(emitted_code);
                fs::write(
                    &out_path.with_extension(format!("{}.as", decompiled_action.id)),
                    formatted,
                )
                .unwrap();
                if (action_size + 4) % 4 != 0 {
                    reader.read_slice(4 - (action_size as usize + 4) % 4);
                }
            }
        }
    }
}
