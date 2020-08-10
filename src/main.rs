mod command;

use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8756")?;
    eprintln!("listening on {}", listener.local_addr()?);
    for tcp in listener.incoming() {
        let tcp = tcp?;
        // spawn a thread so that we can handle multiple connections concurrently
        std::thread::spawn(|| {
            if let Err(e) = cli(tcp) {
                eprintln!("error in cli: {}", e);
            }
        });
    }
    Ok(())
}

fn cli(tcp: TcpStream) -> anyhow::Result<()> {
    let mut tcp = BufReader::new(tcp);
    loop {
        // get_ref() b/c BufReader doesn't delegate Write to the inner type
        write!(tcp.get_ref(), "sock> ")?;
        let cmd = match (&mut tcp).lines().next() {
            Some(res) => res?,
            _ => break,
        };

        // special cases, so that we don't get a clap error when there's an
        // empty line and so that we can use ctrl-d to exit
        match cmd.trim() {
            "" => continue,
            // EOT, ^D
            "\\x04" => break,
            _ => {}
        }

        let parse_result = command::parse(&cmd);

        // If there was a parse error, post the error text back to the client
        match parse_result {
            Ok(command::ParsedCommand::Boss(cmd)) => {
                let cmd_output = execute_command(cmd);
                writeln!(tcp.get_ref(), "{}", cmd_output)?;
            }
            Ok(command::ParsedCommand::Exit) => break,
            Err(e) => {
                writeln!(tcp.get_ref(), "{}", e)?;
            }
        }
    }
    Ok(())
}

fn execute_command(cmd: BossCommand) -> impl std::fmt::Display {
    format!("ran command: {:?}", cmd)
}

#[derive(Debug)]
pub enum BossCommand {
    WifiAdd {
        ssid: String,
        password: Option<String>,
    },
    WifiDelete(String),
    WifiScan,
    Configure(String, String),
    GetConfig(String),
}
