use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet, Styled};

pub fn setup_ch_theme() {
    let mut theme = RenderConfig::default();

    // prompts and prefixes
    theme.prompt = StyleSheet::new()
        .with_fg(Color::LightCyan)
        .with_attr(Attributes::BOLD);
    theme.prompt_prefix = Styled::new("Sonata Prompt").with_fg(Color::LightMagenta);
    theme.answered_prompt_prefix = Styled::new("Sonata Success").with_fg(Color::LightGreen);
    theme.help_message = StyleSheet::new().with_fg(Color::DarkGrey);

    // options selection
    theme.option = StyleSheet::new().with_fg(Color::White);
    theme.highlighted_option_prefix = Styled::new("❯ ").with_fg(Color::LightMagenta);

    inquire::set_global_render_config(theme);
}

pub fn print_logo() {
    let logo = include_str!("../../assets/logo.txt");
    let no_tabs_logo = logo.replace("\t", "    ");
    let clean_logo = no_tabs_logo.replace("\r\n", "\n");
    let trimmed_logo = clean_logo.trim_matches(|c| c == '\n' || c == '\r'); //bloat ._.

    println!("\x1B[96m{}\x1B[0m\n", trimmed_logo);
}
