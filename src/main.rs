use clap::{Parser, Subcommand};

use oanda_cli::{Config, OandaClient, commands};

#[derive(Parser)]
#[command(name = "oanda", about = "CLI for the OANDA v20 REST API")]
struct Cli {
    /// Bearer token (or set OANDA_TOKEN env var)
    #[arg(long, global = true)]
    token: Option<String>,

    /// Account ID (or set OANDA_ACCOUNT_ID env var)
    #[arg(long, global = true)]
    account_id: Option<String>,

    /// Environment: "practice" or "live" (default: practice)
    #[arg(long, global = true)]
    environment: Option<String>,

    /// Accept-Datetime-Format header: "RFC3339" or "UNIX"
    #[arg(long, global = true)]
    datetime_format: Option<String>,

    /// Pretty-print JSON output
    #[arg(long, global = true)]
    pretty: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Account operations
    Account {
        #[command(subcommand)]
        cmd: commands::account::AccountCommand,
    },
    /// Instrument operations
    Instrument {
        #[command(subcommand)]
        cmd: commands::instrument::InstrumentCommand,
    },
    /// Order operations
    Order {
        #[command(subcommand)]
        cmd: commands::order::OrderCommand,
    },
    /// Trade operations
    Trade {
        #[command(subcommand)]
        cmd: commands::trade::TradeCommand,
    },
    /// Position operations
    Position {
        #[command(subcommand)]
        cmd: commands::position::PositionCommand,
    },
    /// Pricing operations
    Pricing {
        #[command(subcommand)]
        cmd: commands::pricing::PricingCommand,
    },
    /// Transaction operations
    Transaction {
        #[command(subcommand)]
        cmd: commands::transaction::TransactionCommand,
    },
    /// Unofficial OANDA Labs API operations
    Labs {
        #[command(subcommand)]
        cmd: commands::labs::LabsCommand,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config = match Config::from_args(
        cli.token,
        cli.account_id,
        cli.environment,
        cli.datetime_format,
        cli.pretty,
    ) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let result = match cli.command {
        Command::Labs { cmd } => commands::labs::execute(config.pretty, cmd).await,
        cmd => {
            let client = match OandaClient::new(&config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            };
            match cmd {
                Command::Account { cmd } => commands::account::execute(&client, &config, cmd).await,
                Command::Instrument { cmd } => commands::instrument::execute(&client, cmd).await,
                Command::Order { cmd } => commands::order::execute(&client, &config, cmd).await,
                Command::Trade { cmd } => commands::trade::execute(&client, &config, cmd).await,
                Command::Position { cmd } => {
                    commands::position::execute(&client, &config, cmd).await
                }
                Command::Pricing { cmd } => commands::pricing::execute(&client, &config, cmd).await,
                Command::Transaction { cmd } => {
                    commands::transaction::execute(&client, &config, cmd).await
                }
                Command::Labs { .. } => unreachable!(),
            }
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
