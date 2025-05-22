mod machine;
mod parser;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// File path describing the DFA machine
    machine_file_path: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Run a DFA machine
    Dfa {
        /// Input string to be processed by the DFA
        input: String,
    },
    /// Run a NFA machine
    Nfa {
        /// Input string to be processed by the NFA
        input: String,
    },
    /// Run a PDA machine
    Pda {
        /// Input string to be processed by the PDA
        input: String,
    },
    /// Run a Turing Machine
    Tm {
        /// Input string to be processed by the Turing Machine
        input: String,
    },
}

fn handle_cli(cli: Cli, src: &'static str) -> miette::Result<()> {
    let lexed = parser::Parser::lex(src)?;
    let parsed = parser::Parser::parse(src, lexed)?;
    match cli.command {
        Commands::Dfa { input } => {
            use machine::dfa;
            let dfa_info = dfa::Info::new(parsed, src)?;
            let dfa = dfa::Machine::new(dfa_info);
            let accepted = dfa.run(&input);

            if accepted {
                println!("Input is ACCEPTED");
            } else {
                println!("Input is REJECTED");
            }
        }
        Commands::Nfa { input } => {
            use machine::nfa;
            let nfa_info = nfa::Info::new(parsed, src)?;
            let nfa = nfa::Machine::new(nfa_info);
            let accepted = nfa.run(&input);

            if accepted {
                println!("Input is ACCEPTED");
            } else {
                println!("Input is REJECTED");
            }
        }
        Commands::Pda { input } => {
            use machine::pda;
            let pda_info = pda::Info::new(parsed, src)?;
            let pda = pda::Machine::new(pda_info);
            let accepted = pda.run(&input);

            if accepted {
                println!("Input is ACCEPTED");
            } else {
                println!("Input is REJECTED");
            }
        }
        Commands::Tm { input } => {
            use machine::tm;
            let tm_info = tm::Info::new(parsed, src)?;
            let tm = tm::Machine::new(tm_info);
            let accepted = tm.run(&input);

            if accepted {
                println!("Input is ACCEPTED");
            } else {
                println!("Input is REJECTED");
            }
        }
    }

    Ok(())
}

fn main() -> miette::Result<()> {
    let cli = <Cli as clap::Parser>::parse();
    let src =
        std::fs::read_to_string(cli.machine_file_path.clone()).expect("Failed to open input file");
    if src.is_empty() {
        println!("Input file is empty");
        return Ok(());
    }

    let path = cli.machine_file_path.clone();
    let src = src.leak();

    handle_cli(cli, src)
        .map_err(|report| report.with_source_code(miette::NamedSource::new(path, &*src)))?;

    Ok(())
}
