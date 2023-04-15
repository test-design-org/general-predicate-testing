#![warn(
    clippy::all,
    // clippy::restriction,
    // clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
)]

use clap::Parser;
use gpt_common::{
    dto::NTupleSingleInterval,
    generate_tests_for_gpt_input,
    graph_reduction::{create_graph, monke::run_monke},
};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    Run(Run),
}

/// Read the input GPT file and generate test cases
#[derive(Parser, Debug)]
struct Run {
    /// Don't print the generated test cases
    #[arg(long)]
    no_show: bool,

    /// Input GPT file path
    file_path: String,
}

fn run(_cli: &Cli, cmd: &Run) -> Result<(), Box<dyn std::error::Error>> {
    let input = std::fs::read_to_string(&cmd.file_path)
        .map_err(|e| format!("Error while reading file {}: {}", &cmd.file_path, e))?;

    let test_cases = generate_tests_for_gpt_input(&input)?;

    let test_cases_json = serde_json::to_string(&test_cases)?;
    println!("Test cases:");
    if !cmd.no_show {
        println!("{}", test_cases_json);
    }
    println!("Number of test cases: {}", test_cases.len());

    let ntuple_graph = create_graph(&test_cases);
    let monked_graph = run_monke(&ntuple_graph);
    let monked_test_cases = monked_graph
        .node_weights()
        .cloned()
        .map(|x| *x)
        .collect::<Vec<NTupleSingleInterval>>();

    let monke_json = serde_json::to_string(&monked_test_cases)?;

    println!("\nAfter running MONKE:");
    if !cmd.no_show {
        println!("{}", monke_json);
    }
    println!("Number of test cases: {}", monked_test_cases.len());

    Ok(())
}

pub fn main() {
    let args = Cli::parse();

    let result = match &args.command {
        Command::Run(cmd) => run(&args, cmd),
    };

    match result {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }

    // let test_cases = match generate_tests_for_gpt_input(input3) {
    //     Ok(test_cases) => test_cases,
    //     Err(e) => panic!("Error: {}", e),
    // };

    // println!("{:#?}", test_cases);
    // println!("Number of test cases: {}", test_cases.len());

    // let ntuple_graph = create_graph(&test_cases);
    // let monked_graph = run_monke(&ntuple_graph);
    // let monked_test_cases = monked_graph
    //     .node_weights()
    //     .cloned()
    //     .collect::<Vec<NTupleSingleInterval>>();

    // println!("After running MONKE:");
    // println!("{:#?}", monked_test_cases);
    // println!("Number of test cases: {}", monked_test_cases.len());
}
