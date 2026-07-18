use clap::{Parser, Subcommand};

use oanda_cli::{Config, OandaError, OandaResult, commands};

#[derive(Parser)]
#[command(
    name = "oanda",
    version,
    about = "CLI for the OANDA v20 REST API",
    long_about = "A safe, machine-friendly CLI for the OANDA v20 REST API. Successful responses are JSON; streams are newline-delimited JSON."
)]
struct Cli {
    /// Bearer token (or set OANDA_ACCESS_TOKEN)
    #[arg(long, global = true, value_name = "TOKEN")]
    token: Option<String>,

    /// Account ID (or set OANDA_ACCOUNT_ID)
    #[arg(long, global = true)]
    account_id: Option<String>,

    /// Account environment (or set OANDA_ENVIRONMENT)
    #[arg(long, global = true, value_parser = ["practice", "live"])]
    environment: Option<String>,

    /// Accept-Datetime-Format header: RFC3339 or UNIX
    #[arg(long, global = true, value_parser = ["RFC3339", "UNIX"])]
    datetime_format: Option<String>,

    /// Pretty-print JSON output
    #[arg(long, global = true)]
    pretty: bool,

    /// Validate and print a mutation without sending it
    #[arg(long, global = true)]
    dry_run: bool,

    /// Non-streaming request timeout in seconds
    #[arg(long, global = true)]
    request_timeout_secs: Option<u64>,

    /// Connection timeout in seconds
    #[arg(long, global = true)]
    connect_timeout_secs: Option<u64>,

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
    /// Gather instrument metadata, price, position, orders, and trades
    Inspect {
        /// Instrument name, for example EUR_USD
        instrument: String,
    },
    /// Unofficial OANDA Labs API operations
    Labs {
        #[command(subcommand)]
        cmd: commands::labs::LabsCommand,
    },
    /// Print the machine-readable command schema
    Schema {
        /// Emit JSON (accepted for explicit machine use; schema output is always JSON)
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            if let Err(error) = error.print() {
                fail(OandaError::Io(error));
            }
            return;
        }
        Err(error) => fail(OandaError::Validation(error.to_string())),
    };
    if let Err(error) = run(cli).await {
        fail(error);
    }
}

fn fail(error: OandaError) -> ! {
    eprintln!("{}", error.structured());
    std::process::exit(error.exit_code());
}

async fn run(cli: Cli) -> OandaResult<()> {
    let Cli {
        token,
        account_id,
        environment,
        datetime_format,
        pretty,
        dry_run,
        request_timeout_secs,
        connect_timeout_secs,
        command,
    } = cli;
    match command {
        Command::Schema { json: _ } => {
            let output = if pretty {
                serde_json::to_string_pretty(&commands::schema::document())
            } else {
                serde_json::to_string(&commands::schema::document())
            }
            .map_err(OandaError::Json)?;
            println!("{output}");
            Ok(())
        }
        Command::Labs { cmd } => commands::labs::execute(pretty, cmd).await,
        command => {
            let config =
                Config::from_args(token, account_id, environment, datetime_format, pretty)?
                    .with_dry_run(dry_run)
                    .with_timeouts(request_timeout_secs, connect_timeout_secs)?;
            let client = oanda_cli::OandaClient::new(&config)?;
            execute(&client, &config, command).await
        }
    }
}

async fn execute(
    client: &oanda_cli::OandaClient,
    config: &Config,
    command: Command,
) -> OandaResult<()> {
    match command {
        Command::Account { cmd } => commands::account::execute(client, config, cmd).await,
        Command::Instrument { cmd } => commands::instrument::execute(client, cmd).await,
        Command::Order { cmd } => commands::order::execute(client, config, cmd).await,
        Command::Trade { cmd } => commands::trade::execute(client, config, cmd).await,
        Command::Position { cmd } => commands::position::execute(client, config, cmd).await,
        Command::Pricing { cmd } => commands::pricing::execute(client, config, cmd).await,
        Command::Transaction { cmd } => commands::transaction::execute(client, config, cmd).await,
        Command::Inspect { instrument } => {
            commands::inspect::execute(client, config, instrument).await
        }
        Command::Labs { .. } | Command::Schema { .. } => unreachable!(),
    }
}
