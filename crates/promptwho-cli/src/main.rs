use anyhow::Result;
use bat::PrettyPrinter;
use clap::builder::Styles;
use clap::{Args, Parser, Subcommand, builder::styling::AnsiColor};
use promptwho_core::PromptwhoConfig;
use promptwho_core::telemetry::init_tracing;
use promptwho_protocol::PluginSource;
use promptwho_server::run;
use promptwho_watcher::{GitWatcher, HttpEventEmitter, WatcherConfig};
use std::{fs, path::PathBuf, time::Duration};

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

fn resolve_git_dir(path: PathBuf) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()?.join(path)
    };

    Ok(fs::canonicalize(path)?)
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
    /// Watch local sources and emit promptwho events.
    Watch(WatchArgs),
    /// Run a query agains the promptwho server.
    Query,
}

#[derive(Debug, Clone, Args)]
struct WatchArgs {
    #[command(subcommand)]
    command: WatchCommand,
}

#[derive(Debug, Clone, Subcommand)]
enum WatchCommand {
    /// Watch a git repository for new commits.
    Git(GitWatchArgs),
}

#[derive(Debug, Clone, Args)]
struct GitWatchArgs {
    #[arg(
        short = 'g',
        long = "git-dir",
        value_name = "PATH",
        default_value = "."
    )]
    /// Path to the git working tree or repository to watch.
    git_dir: PathBuf,

    #[arg(short = 'u', long, value_name = "URL")]
    /// Override the promptwho server base URL.
    server_url: Option<String>,

    #[arg(short = 'i', long, value_name = "MILLIS", default_value_t = 2000)]
    /// Poll interval in milliseconds.
    poll_interval_ms: u64,

    #[arg(short = 'e', long, default_value_t = false)]
    /// Emit the current HEAD immediately instead of waiting for a new commit.
    emit_existing_head: bool,
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
        Command::Watch(args) => match args.command {
            WatchCommand::Git(args) => {
                let git_dir = resolve_git_dir(args.git_dir)?;
                let server_url = args.server_url.unwrap_or_else(|| {
                    format!("http://{}:{}", config.server.host, config.server.port)
                });
                let emitter = HttpEventEmitter::new(server_url);
                let mut watcher = GitWatcher::new(
                    WatcherConfig {
                        repo_path: git_dir,
                        poll_interval: Duration::from_millis(args.poll_interval_ms),
                        emit_existing_head: args.emit_existing_head,
                        project_id: None,
                        project_name: None,
                        checkpoint_path: None,
                        source: PluginSource {
                            plugin: "promptwho-cli".to_string(),
                            plugin_version: env!("CARGO_PKG_VERSION").to_string(),
                            runtime: "rust".to_string(),
                        },
                    },
                    emitter,
                )?;

                watcher.run().await?;
            }
        },
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
