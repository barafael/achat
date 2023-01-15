use std::net::SocketAddr;

use clap::Parser;

/// Command Line Arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Arguments {
    /// Address to listen on.
    #[clap(short, long, value_parser, default_value = "127.0.0.1:8080")]
    pub address: SocketAddr,

    /// Address to publish console events on.
    #[clap(short, long, value_parser)]
    pub console: Option<SocketAddr>,
}
