use clap::Args;
use clap::Parser;
use clap::Subcommand;
use std::path::PathBuf;

use tools::utils;

mod tools;

#[derive(Parser)]
#[command(
    disable_help_flag = true,
    disable_help_subcommand = true,
    disable_version_flag = true
)]
#[command(arg_required_else_help = true)]
#[command(about=utils::cli_about())]
#[command(styles=utils::cli_styles())]
struct App {
    /// Показать версию и выйти
    #[arg(short, long, default_value_t = false)]
    version: bool,

    /// Commands
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Работа с эмулятором
    #[command(short_flag = 'e')]
    Emulator(EmulatorArgs),

    /// Сервисы Build Tools
    #[command(short_flag = 's')]
    Services(ServicesArgs),
}

/// Классическая командная строка
#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct EmulatorArgs {
    /// Открыть эмулятор
    #[arg(short, long, default_value_t = false)]
    open: bool,

    /// Закрыть эмулятор
    #[arg(short, long, default_value_t = false)]
    close: bool,

    /// Установить пакет
    #[arg(short, long, value_name = "path")]
    install: Option<PathBuf>,

    /// Запустить пакет
    #[arg(short, long, value_name = "package")]
    run: Option<String>,

    /// Показать это сообщение и выйти
    #[clap(short='h', long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,
}

/// Классическая командная строка
#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct ServicesArgs {
    /// Подпись пакета публичным ключем
    #[arg(short, long, value_name = "path")]
    rpmsign: Option<PathBuf>,

    /// Создать sysroot (aarch64, armv7hl, x86_64)
    #[arg(short, long, default_value_t = false)]
    sysroot: bool,

    /// Показать это сообщение и выйти
    #[clap(short='h', long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,
}

fn main() {
    if App::parse().version {
        println!("{}", utils::cli_version());
        return;
    }
    match App::parse().command.unwrap() {
        Commands::Emulator(arg) => {
            if arg.open {
                println!("Emulator open: true");
                return;
            }
            if arg.close {
                println!("Emulator close: true");
                return;
            }
            if let Some(path) = arg.install {
                println!("Emulator install: {}", path.to_string_lossy());
                return;
            }
            if let Some(package) = arg.run {
                println!("Emulator run: {}", package);
                return;
            }
        },
        Commands::Services(arg) => {
            if let Some(path) = arg.rpmsign {
                println!("Emulator rpmsign: {}", path.to_string_lossy());
                return;
            }
            if arg.sysroot {
                println!("Services sysroot: true");
                return;
            }
        },
    }
}
