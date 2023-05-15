#![warn(
    clippy::all,
    // clippy::restriction,
    // clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
)]

use core::fmt;

use clap::{Parser, ValueEnum};
use gpt_common::{
    and_reduce_gpt_input,
    dto::NTupleSingleInterval,
    generate_tests_for_gpt_input,
    graph_reduction::{
        create_graph,
        least_losing_components::run_least_losing_components,
        least_losing_edges::{run_least_losing_edges, run_most_losing_edges},
        least_losing_nodes_reachable::run_least_losing_nodes_reachable,
        monke::run_monke,
    },
    parser::parse_gpt_to_ir,
};
use itertools::Itertools;

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
    AndReduce(AndReduce),
}

/// Read the input GPT file and generate test cases
#[derive(Parser, Debug)]
struct Run {
    /// Don't print the generated test cases
    #[arg(long)]
    no_show: bool,

    // Format to show the generated test cases in
    #[arg(long, value_enum, default_value_t = ShowFormat::Json)]
    show_format: ShowFormat,

    /// Graph reduction algorithm to use
    #[arg(short, long, value_enum, default_value_t = Algo::Monke)]
    algo: Algo,

    /// Input GPT file path
    file_path: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Algo {
    /// Don't do any graph reduction
    None,
    /// MONKE
    Monke,
    /// Least Losing Nodes Reachable
    Llnr,
    /// Least Losing Edges
    Lle,
    /// Most Losing Edges
    Mle,
    /// Least Losing Components
    Llc,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ShowFormat {
    /// JSON
    Json,
    /// Typst table
    Typst,
}

#[derive(Parser, Debug)]
struct AndReduce {
    /// Input GPT file path
    file_path: String,
}

impl fmt::Display for Algo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "None",
                Self::Monke => "MONKE",
                Self::Llnr => "Least Losing Nodes Reachable",
                Self::Lle => "Least Losing Edges",
                Self::Mle => "Most Losing Edges",
                Self::Llc => "Least Losing Components",
            }
        )
    }
}

fn print_typst_format(test_cases: &[NTupleSingleInterval]) {
    for test_case in test_cases.iter() {
        let middle = test_case
            .iter()
            .sorted_by_key(|(x, _)| *x)
            .map(|(k, v)| {
                format!(
                    "{k}: ${}$",
                    match v {
                        gpt_common::dto::Output::MissingVariable => "*".to_owned(),
                        gpt_common::dto::Output::Bool(x) => format!("{x}"),
                        gpt_common::dto::Output::Interval(x) => format!("{x}"),
                    }
                )
            })
            .join(", ")
            .replace("Inf", "infinity");

        println!("[M], [{middle}], [],")
    }
}

fn show(
    test_cases: &[NTupleSingleInterval],
    show_format: ShowFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match show_format {
        ShowFormat::Json => {
            let test_cases_json = serde_json::to_string(&test_cases)?;
            println!("{}", test_cases_json);

            Ok(())
        }
        ShowFormat::Typst => {
            print_typst_format(test_cases);
            Ok(())
        }
    }
}

fn run(_cli: &Cli, cmd: &Run) -> Result<(), Box<dyn std::error::Error>> {
    let input = std::fs::read_to_string(&cmd.file_path)
        .map_err(|e| format!("Error while reading file {}: {}", &cmd.file_path, e))?;

    let test_cases = generate_tests_for_gpt_input(&input)?;

    println!("Test cases:");
    if !cmd.no_show {
        show(&test_cases, cmd.show_format)?;
    }
    println!("Number of test cases: {}", test_cases.len());

    let ntuple_graph = create_graph(&test_cases);

    println!(
        "Number of edges in initial graph: {}",
        ntuple_graph.edge_count()
    );

    let reduced_graph = match cmd.algo {
        Algo::None => ntuple_graph,
        Algo::Monke => run_monke(&ntuple_graph),
        Algo::Llnr => run_least_losing_nodes_reachable(&ntuple_graph),
        Algo::Lle => run_least_losing_edges(&ntuple_graph),
        Algo::Mle => run_most_losing_edges(&ntuple_graph),
        Algo::Llc => run_least_losing_components(&ntuple_graph),
    };

    let reduced_test_cases = reduced_graph
        .node_weights()
        .cloned()
        .map(|x| *x)
        .collect::<Vec<NTupleSingleInterval>>();

    println!("\nAfter running {}:", cmd.algo);
    if !cmd.no_show {
        show(&reduced_test_cases, cmd.show_format)?;
    }
    println!("Number of test cases: {}", reduced_test_cases.len());

    Ok(())
}

fn and_reduce(_cli: &Cli, cmd: &AndReduce) -> Result<(), Box<dyn std::error::Error>> {
    let input = std::fs::read_to_string(&cmd.file_path)
        .map_err(|e| format!("Error while reading file {}: {}", &cmd.file_path, e))?;

    let ir = and_reduce_gpt_input(&input)?;

    for predicate in ir.into_iter().flat_map(|feature| feature.predicates) {
        println!("Predicate: {predicate}");
        println!("Reduced predicate {:#?}", predicate.reduce());
        println!("Conjunction of conjunctions:");
        for conjunctions in predicate.conjunction_of_conditions() {
            println!("{}", conjunctions.iter().join(" && "));
        }
    }

    Ok(())
}

pub fn main() {
    let args = Cli::parse();

    let result = match &args.command {
        Command::Run(cmd) => run(&args, cmd),
        Command::AndReduce(cmd) => and_reduce(&args, cmd),
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
