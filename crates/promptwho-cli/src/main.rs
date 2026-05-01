use anyhow::Result;
use bat::PrettyPrinter;
use clap::builder::Styles;
use clap::{Args, Parser, Subcommand, builder::styling::AnsiColor};
use promptwho_core::PromptwhoConfig;
use promptwho_core::telemetry::init_tracing;
use promptwho_server::run;

fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().bold().underline())
        .usage(AnsiColor::Yellow.on_default().bold().underline())
        .placeholder(AnsiColor::White.on_default().dimmed())
        .literal(AnsiColor::BrightBlue.on_default().bold())
        .invalid(AnsiColor::Red.on_default().bold().italic())
        .context(AnsiColor::White.on_default())
        .context_value(AnsiColor::White.on_default().dimmed())
}

#[derive(Debug, Parser)]
#[command(name = "promptwho")]
#[command(about = "Inspect conversation and code attribution data", styles=get_styles())]
struct Cli {
    #[arg(long, global = true, env = "PROMPTWHO_CONFIG")]
    /// Override default path to configuration file. Can be set via the PROMPTWHO_CONFIG environment variable.
    config: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check for common issues and provide diagnostics.
    Doctor,
    /// Print the current configuration in a human-readable format
    Config(ConfigArgs),
    /// Start the promptwho server. This will run the server and block until it is stopped.
    Serve,
    /// Run a query agains the promptwho server.
    Query,
}

#[derive(Debug, Clone, Args)]
struct ConfigArgs {
    #[arg(
        short,
        long,
        global = false,
        env = "PROMPTWHO_COLORS",
        default_value_t = false
    )]
    /// Whether to use colors in the output. Can be set via the PROMPTWHO_COLORS environment
    /// variable.
    colors: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = PromptwhoConfig::load(cli.config.clone());
    init_tracing(&config);
    match cli.command {
        Command::Doctor => {
            todo!("Implement doctor command to check for common issues and provide diagnostics");
        }
        Command::Serve => {
            let r = run(&config).await;
            tracing::info!("Promptwho server exited with result: {:?}", &r);
            r?
        }
        Command::Config(args) => {
            if args.colors {
                let string = toml::to_string(&config)?;
                PrettyPrinter::new()
                    .input_from_bytes(string.as_bytes())
                    .language("toml")
                    .print()?;
            } else {
                println!("{}", toml::to_string_pretty(&config)?);
            }
        }
        Command::Query => {
            todo!("Implement query command to run a query against the promptwho server");
        }
    }
    Ok(())
}
