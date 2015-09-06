extern crate algorithmia;
extern crate chan;
extern crate docopt;
extern crate rustc_serialize;
extern crate toml;

use algorithmia::{Algorithmia, Url};
use std::env;
use std::vec::IntoIter;
use toml::Value;

macro_rules! die {
    ($fmt:expr) => ({
        print!(concat!($fmt, "\n"));
        ::std::process::exit(1)
    });
    ($fmt:expr, $($arg:tt)*) => ({
        print!(concat!($fmt, "\n"), $($arg)*);
        ::std::process::exit(1)
    });
}

mod data;
mod algo;
mod auth;

static USAGE: &'static str = "
CLI for interacting with Algorithmia

Usage:
  algo [<cmd>] [options] [<args>...]
  algo [<cmd>] [--help]

General commands include:
  auth      Configure authentication

Algorithm commands include:
  run       Runs an algorithm

Data commands include
  ls        List contents of a data directory
  mkdir     Create a data directory
  rmdir     Delete a data directory
  rm        Remove a file from a data directory
  cp        Copy file(s) to or from a data directory
  cat       Concatenate and print file(s) in a data directory

Global options:
  --help                Prints the help for a particular command
  --profile <name>      Run a particular command for the specified profile
";

/* TODO: Add support for:

Note: Add Option [--profile <profile>]

Algorithm commands include:
  view      View algorithm details (e.g. cost)
  clone     Clone an algorithm (wrapper around git clone)
  fork      Fork an algorithm

Data commands include:
  download  Download file(s) from a collection
  rm        Delete file(s) in a collection
  chmod     Change permissions on a collection
*/

fn print_usage() -> ! {
    die!("{}", USAGE)
}

#[derive(RustcDecodable, Debug)]
struct MainArgs {
    arg_args: Vec<String>,
    arg_cmd: Option<String>,
    flag_h: bool,
}

fn main() {
    let mut args = env::args().peekable();
    let mut cmd_args: Vec<String> = Vec::new();
    let mut profile = "default".to_string();

    // Search for global options, push everything else onto cmd_args
    while let Some(arg) = args.next() {
        match &*arg {
            "--help" => {
                // grab one more arg in-case --help preceded <cmd>
                cmd_args.push(args.next().unwrap_or_default());
                print_cmd_usage(cmd_args.get(1));
            },
            "--profile" => {
                profile = args.next().unwrap_or(profile.to_string())
            },
            _ => cmd_args.push(arg),
        }
    };

    if cmd_args.len() < 2 {
        print_cmd_usage(None);
    } else {
        run(cmd_args, &*profile);
    }
}

pub fn get_config_path() -> String {
    if cfg!(windows) {
        format!("{}/algorithmia", env::var("LOCALAPPDATA").unwrap())
    } else {
        format!("{}/.algorithmia", env::var("HOME").unwrap())
    }
}

fn init_client(profile: &str) -> Algorithmia {
    match auth::Auth::read_profile(profile.into()) {

        // Use the profile attribute(s)
        Some(p) => match p.get("api_key") {
            Some(&Value::String(ref key)) => match p.get("api_server") {
                Some(&Value::String(ref api)) if api.parse::<Url>().is_ok() => Algorithmia::alt_client(api.parse().unwrap(), &key),
                None => Algorithmia::client(&key),
                _ => die!("{} profile has invalid 'api_server'", profile),
            },
            _ => die!("{} profile has missing or invalid 'api_key'", profile),
        },

        // Fall-back to env var (but only if profile was not specified)
        None => match profile {
            "default" => match env::var("ALGORITHMIA_API_KEY") {
                Ok(ref key) => match env::var("ALGORITHMIA_API") {
                    Ok(ref api) if api.parse::<Url>().is_ok() => Algorithmia::alt_client(api.parse().unwrap(), &**key),
                    Err(_) => Algorithmia::client(&**key),
                    _ => die!("Invalid ALGORITHMIA_API environment variable"),
                },
                Err(_) => die!("Run 'algo auth' or set ALGORITHMIA_API_KEY"),
            },
            _ => die!("{} profile not found. Run 'algo auth {0}'", profile),
        }
    }
}

fn run(args: Vec<String>, profile: &str) {
    let cmd = match args.get(1) {
        Some(c) => c.clone(),
        _ => "run".into(),
    };

    let args_iter = args.into_iter();
    match &*cmd {
        "auth" => auth::Auth::new().cmd_main(args_iter),
        _ => {
            let client = init_client(profile);
            match &*cmd {
                "ls" | "dir" => data::Ls::new(client).cmd_main(args_iter),
                "mkdir" => data::MkDir::new(client).cmd_main(args_iter),
                "rmdir" => data::RmDir::new(client).cmd_main(args_iter),
                "rm" => data::Rm::new(client).cmd_main(args_iter),
                "cp" | "copy" => data::Cp::new(client).cmd_main(args_iter),
                "cat" => data::Cat::new(client).cmd_main(args_iter),
                "run" => algo::Run::new(client).cmd_main(args_iter),
                _ => algo::Run::new(client).cmd_main(args_iter),
            }
        },
    };
}

fn print_cmd_usage(cmd: Option<&String>) -> ! {
    match &**cmd.unwrap_or(&Default::default()) {
        "auth" => auth::Auth::print_usage(),
        "ls" | "dir" => data::Ls::print_usage(),
        "mkdir" => data::MkDir::print_usage(),
        "rmdir" => data::RmDir::print_usage(),
        "rm" => data::Rm::print_usage(),
        "cp" | "copy" => data::Cp::print_usage(),
        "cat" => data::Cat::print_usage(),
        "run" => algo::Run::print_usage(),
        _ => print_usage(),
    };
}


trait CmdRunner {
    fn cmd_main(&self, argv: IntoIter<String>);
    fn get_usage() -> &'static str;

    fn print_usage() -> ! { die!("{}", Self::get_usage()) }
}