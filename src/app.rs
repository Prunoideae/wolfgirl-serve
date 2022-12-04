use std::path::PathBuf;

use clap::{command, Parser};

#[derive(Debug, Parser)]
#[command(name = "wolfgirl-serve")]
#[command(author = "Prunoideae <jlijh@connect.ust.hk>")]
#[command(version = "1.0")]
#[command(about = "Serves static files for static.wolfgirl.moe domain", long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        value_name = "PORT",
        default_value_t = 8080,
        help = "The port which it will be served on"
    )]
    pub port: u16,

    #[arg(value_name = "PATH", help = "The path to be served on server")]
    pub dir: Option<PathBuf>,

    #[arg(
        long,
        short,
        value_name = "ADDRESS",
        help = "The ip the server will listen on"
    )]
    pub addr: Option<String>,

    #[arg(
        long,
        short,
        default_value_t = 16,
        help = "Workers for Rocket to run on"
    )]
    pub workers: u16,
}
