// Manpage-like README does not contain intra_doc_links
#![allow(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

mod commands;
mod datetime;
mod store;

use std::process::ExitCode;

use lexopt::prelude::*;

use crate::commands::{
        ListArgs,
        Task,
        print_global_help,
};

fn main() -> ExitCode {
        if let Err(e) = run() {
                eprintln!("[task]: error: {}", e);
                return ExitCode::FAILURE;
        }
        ExitCode::SUCCESS
}

#[rustfmt::skip]
fn run() -> Result<(), lexopt::Error> {
        let mut parser = lexopt::Parser::from_env();

        let Some(arg) = parser.next()? else { return Task::list(ListArgs::default(), &mut parser) };

        match arg {
                // Global flags
                Short('h') | Long("help")    => { print_global_help(); Ok(()) },
                Short('V') | Long("version") => { println!("tafsk {}", env!("CARGO_PKG_VERSION")); Ok(()) },

                // Explicit subcommands
                Value(val) if val == "add"  => Task::add(&mut parser),
                Value(val) if val == "done" => Task::done(&mut parser),
                Value(val) if val == "init" => Task::init(&mut parser),
                Value(val) if val == "list" => Task::list(ListArgs::default(), &mut parser),

                // Implicit `list`
                flags => {
                        let mut args = ListArgs::default();
                        match flags {
                                // TASK(20251219-010809.f3fe84e5)
                                Short('c') | Long("closed") => args.show_closed = true,
                                Short('g') | Long("global") => args.show_global = true,
                                _ => {
                                        print_global_help();
                                        println!();
                                        return Err(flags.unexpected());
                                }
                        }
                        Task::list(args, &mut parser)
                },
        }
}
