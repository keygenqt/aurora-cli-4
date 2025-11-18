use clap::Args;
use clap::CommandFactory;
use clap::Parser;
use clap::Subcommand;
use std::path::PathBuf;

use crate::features::moc::QtMoc;
use crate::features::rpm::RpmBuild;
use crate::features::sign::RpmSign;
use crate::features::sysroot::Sysroot;
use crate::features::validator::Validator;
use crate::tools::utils;

mod features;
mod tools;

#[derive(Parser)]
#[command(
    disable_help_flag = true,
    disable_help_subcommand = true,
    disable_version_flag = true
)]
#[command(about=utils::cli_about())]
#[command(styles=utils::cli_styles())]
struct App {
    /// Показать версию и выйти
    #[arg(short, long, default_value_t = false)]
    version: bool,

    /// Commands
    #[command(subcommand)]
    command: Option<Commands>,

    /// Показать это сообщение и выйти
    #[arg(short, long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,
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
    /// Валидация rpm пакета
    #[arg(short, long, value_name = "path")]
    validate: Option<PathBuf>,

    /// Подпись пакета публичным ключем
    #[arg(short, long, value_name = "path")]
    keysign: Option<PathBuf>,

    /// Собрать rpm пакет
    #[arg(short, long, value_name = "path")]
    rpmbuild: Option<PathBuf>,

    /// Создать sysroot (aarch64, armv7hl, x86_64)
    #[arg(short, long, default_value_t = false)]
    sysroot: bool,

    /// Генерация mос файлов для Qt
    #[arg(short, long, value_name = "path")]
    moc: Option<PathBuf>,

    /// Показать это сообщение и выйти
    #[clap(short='h', long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,
}

fn main() {
    if App::parse().version {
        println!("{}", utils::cli_version());
        return;
    }
    match App::parse().command {
        Some(command) => match command {
            Commands::Emulator(arg) => {
                if arg.open {
                    println!("@todo coming soon");
                    return;
                }
                if arg.close {
                    println!("@todo coming soon");
                    return;
                }
                if let Some(_) = arg.install {
                    println!("@todo coming soon");
                    return;
                }
                if let Some(_) = arg.run {
                    println!("@todo coming soon");
                    return;
                }
            }
            Commands::Services(arg) => {
                if let Some(path) = arg.validate {
                    match Validator::validate(path) {
                        Ok(output) => print_success!(output),
                        Err(err) => exit_error!(err),
                    }
                    return;
                }
                if let Some(path) = arg.keysign {
                    match RpmSign::sign(path) {
                        Ok(_) => print_success!("rpm sign successfully"),
                        Err(err) => exit_error!(err),
                    }
                    return;
                }
                if let Some(path) = arg.rpmbuild {
                    match RpmBuild::build(path) {
                        Ok(_) => print_success!("rpm build successfully"),
                        Err(err) => exit_error!(err),
                    }
                    return;
                }
                if let Some(path) = arg.moc {
                    match QtMoc::upgrade(path) {
                        Ok(_) => print_success!("moc files successfully created"),
                        Err(err) => exit_error!(err),
                    }
                    return;
                }
                if arg.sysroot {
                    match Sysroot::upgrade() {
                        Ok(_) => {},
                        Err(err) => exit_error!(err),
                    }
                    return;
                }
            }
        },
        None => {
            let _ = App::command().print_help();
        }
    }
}
