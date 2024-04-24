use clap::{Args, Parser, Subcommand};
use pngme::commands;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug)]
struct EncodeArgs {
    file_path: String,
    #[arg(short = 'c', long)]
    chunk_type: String,
    #[arg(short = 'm', long)]
    message: String,
    #[arg(short = 'o', long)]
    output_file: Option<String>,
}

#[derive(Args, Debug)]
struct DecodeArgs {
    file_path: String,
    #[arg(short = 'c', long)]
    chunk_type: String,
}

#[derive(Args, Debug)]
struct RemoveArgs {
    file_path: String,
    #[arg(short = 'c', long)]
    chunk_type: String,
}

#[derive(Args, Debug)]
struct PrintArgs {
    file_path: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Encode a message into a PNG file
    Encode(EncodeArgs),
    /// Decode a message stored in a PNG file
    Decode(DecodeArgs),
    /// Remove a message from PNG file
    Remove(RemoveArgs),
    /// Print a list of PNG chunks that can be searched for messages
    Print(PrintArgs),
}

fn main() -> Result<(), commands::CommandError> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Encode(args) => commands::encode(
            &args.file_path,
            &args.chunk_type,
            &args.message,
            args.output_file.as_deref(),
        ),
        Commands::Decode(args) => commands::decode(&args.file_path, &args.chunk_type),
        Commands::Remove(args) => commands::remove(&args.file_path, &args.chunk_type),
        Commands::Print(args) => commands::print_chunks(&args.file_path),
    }
}
