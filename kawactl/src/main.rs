use anyhow::{Context, Result};
use clap::{Parser};
use kawactl::{request::GetReq, command::*};
use kawactl::{request::Request, Response, socket_path};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}



fn main() -> Result<()> {
    let req = match Cli::parse().cmd {
        Cmd::Daemon => {
            let err = std::process::Command::new("kawa").exec();
            return Err(err).context("impossible de lancer kawa");
        }
        Cmd::Get { what: GetCmd::Outputs} => Request::Get { what: GetReq::Outputs },
        Cmd::Version => Request::Version,
        Cmd::Quit => Request::Quit,
        Cmd::Exit => Request::Exit,
        _ => Request::Close,
    };

    let mut s = UnixStream::connect(socket_path()).context("Is Kawa Running ?")?;
    writeln!(s, "{}", serde_json::to_string(&req)?)?;
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line)?;
    match serde_json::from_str::<Response>(&line)? {
        // just printing the result for now, testing
        Response::Ok { data } => println!("{}", serde_json::to_string_pretty(&data)?),
        Response::Err { message } => {
            eprintln!("erreur: {message}");
            std::process::exit(1)
        }
    }
    Ok(())
}
