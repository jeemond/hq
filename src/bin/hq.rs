use std::{
    error::Error,
    io::{self, Read, Write},
};

use clap::{Parser, Subcommand};
use hcl_edit::{expr::Expression, structure::Body};
use hq_rs::{parse_filter, query, write};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(value_name = "filter", help = "HCL filter expression")]
    filter: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Read value from HCL (default)")]
    Read,
    #[command(about = "Write value into HCL")]
    Write {
        #[arg(required = true, help = "Value to write into HCL")]
        value: String,
    },
}

impl Default for Command {
    fn default() -> Self {
        Self::Read
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut stdin = io::stdin();
    let mut buf = String::new();
    stdin.read_to_string(&mut buf)?;

    match args.command {
        None | Some(Command::Read) => {
            let body: hcl::Body = hcl::from_str(&buf)?;

            if let Some(filter) = args.filter {
                let mut fields = parse_filter(&filter)?;
                let query_results = query(&mut fields, &body);
                for query_result in query_results {
                    let s = query_result.to_string()?;
                    print!("{s}");
                    io::stdout().flush()?;
                }
            } else {
                println!("HCL from stdin contained:");
                println!(" * {} top-level attribute(s)", body.attributes().count());
                println!(" * {} top-level block(s)", body.blocks().count());
            }
        }
        Some(Command::Write { value }) => {
            let mut body: Body = buf.parse()?;

            let expr: Expression = value.parse()?;
            if let Some(filter) = args.filter {
                let fields = parse_filter(&filter)?;
                write(fields, &mut body, &expr);
                print!("{body}");
                io::stdout().flush()?;
            } else {
                print!("{expr}");
                io::stdout().flush()?;
            }
        }
    }

    Ok(())
}
