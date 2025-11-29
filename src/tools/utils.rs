use clap::builder::Styles;
use colored::Colorize;

pub fn cli_version() -> &'static str {
    "0.1.0"
}

pub fn cli_about() -> String {
    format!(
        r#"

{} - приложение упрощающие работу с инфраструктурой ОС Аврора.

{}"#,
        "Aurora CLI".bright_green().bold(),
        "Это сторонний инструмент, написанный энтузиастами!".italic()
    )
}

pub fn cli_styles() -> Styles {
    Styles::styled()
        .header(clap_cargo::style::HEADER)
        .usage(clap_cargo::style::USAGE)
        .literal(clap_cargo::style::LITERAL)
        .placeholder(clap_cargo::style::PLACEHOLDER)
        .error(clap_cargo::style::ERROR)
        .valid(clap_cargo::style::VALID)
        .invalid(clap_cargo::style::INVALID)
}
