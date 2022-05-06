mod backend;
mod config;

use backend::Backend;
use clap::Parser;
use log::debug;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::fs::File;
use tower_lsp::{LspService, Server};

// -------------------------------------------------------------------------------------------------
// Opt
// -------------------------------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[clap(name = "svls")]
#[clap(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
pub struct Opt {
    /// Debug mode
    #[clap(short = 'd', long = "debug")]
    pub debug: bool,
}

// -------------------------------------------------------------------------------------------------
// Main
// -------------------------------------------------------------------------------------------------

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let opt: Opt = Parser::parse();

    if opt.debug {
        let _ = WriteLogger::init(
            LevelFilter::Debug,
            Config::default(),
            File::create("svls.log").unwrap(),
        );
    }

    debug!("start");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, messages) = LspService::new(Backend::new);
    Server::new(stdin, stdout, messages)
        .serve(service)
        .await;
}
