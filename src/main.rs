mod backend;
mod config;

use backend::Backend;
use log::debug;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::fs::File;
use structopt::{clap, StructOpt};
use tower_lsp::{LspService, Server};

// -------------------------------------------------------------------------------------------------
// Opt
// -------------------------------------------------------------------------------------------------

#[derive(Debug, StructOpt)]
#[structopt(name = "svls")]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct Opt {
    /// Debug mode
    #[structopt(short = "d", long = "debug")]
    pub debug: bool,
}

// -------------------------------------------------------------------------------------------------
// Main
// -------------------------------------------------------------------------------------------------

fn main() {
    let opt = Opt::from_args();

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

    let (service, messages) = LspService::new(Backend::default());
    let handle = service.close_handle();
    let server = Server::new(stdin, stdout)
        .interleave(messages)
        .serve(service);

    tokio::run(handle.run_until_exit(server));
}
