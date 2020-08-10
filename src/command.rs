use crate::BossCommand;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt)]
enum Wifi {
    Add {
        ssid: String,
        password: Option<String>,
    },
    Delete {
        ssid: String,
    },
    Scan,
}

struct ConfigKey(String);
impl FromStr for ConfigKey {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> anyhow::Result<Self> {
        if s.chars().all(|c| c.is_ascii_alphanumeric()) {
            Ok(Self(s.to_owned()))
        } else {
            Err(anyhow::anyhow!("config key {:?} must be alphanumeric", s))
        }
    }
}

#[derive(StructOpt)]
enum Config {
    Set { key: ConfigKey, value: String },
    Get { key: ConfigKey },
}

#[derive(StructOpt)]
#[structopt(
    name = "socket-commander",
    // NoBinaryName means that clap won't expect the first argument in the
    // list to be the cli binary's path
    setting(clap::AppSettings::NoBinaryName),
    global_setting(clap::AppSettings::ColoredHelp)
)]
enum SockCommand {
    Wifi(Wifi),
    Conf(Config),
    Exit,
}

pub enum ParsedCommand {
    Boss(BossCommand),
    Exit,
}

pub fn parse(command: &str) -> anyhow::Result<ParsedCommand> {
    let words = shell_words::split(command)?;

    // StructOpt has a "safe" version as well; errors from this include invalid commands
    // but also just `--help` invocations; it's all fine since we just write!(tcp, "{}", err)
    // and the fmt::Display impl takes care of it all
    let cmd = SockCommand::from_iter_safe(words)?;

    macro_rules! c {
        // have $($args)* in order to handle Command::Foo(foo) or Command::Bar { bar: baz }
        ($cmd:ident$($args:tt)*) => {
            ParsedCommand::Boss(BossCommand::$cmd$($args)*)
        };
    }

    let cmd = match cmd {
        SockCommand::Exit => ParsedCommand::Exit,
        SockCommand::Wifi(wifi) => match wifi {
            Wifi::Add { ssid, password } => c!(WifiAdd { ssid, password }),
            Wifi::Delete { ssid } => c!(WifiDelete(ssid)),
            Wifi::Scan => c!(WifiScan),
        },
        SockCommand::Conf(conf) => match conf {
            Config::Set { key, value } => c!(Configure(key.0, value)),
            Config::Get { key } => c!(GetConfig(key.0)),
        },
        // about 15 more commands in the real version...
    };

    Ok(cmd)
}
