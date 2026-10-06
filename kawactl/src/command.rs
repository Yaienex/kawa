use clap::{Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Subcommand,Serialize,Deserialize,Debug)]
#[serde(tag="cmd", rename_all = "kebab-case")]
pub enum Cmd {
    Daemon,
    Version,

    Get {
        #[command(subcommand)]
        what: GetCmd,
    },

    Quit,
    Exit,
}

#[derive(Subcommand,Serialize,Deserialize,Debug)]
#[serde(tag="get", rename_all = "kebab-case")]
pub enum GetCmd{
    Outputs,
    Windows,
}