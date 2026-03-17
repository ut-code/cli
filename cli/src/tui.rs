use cfonts::{render, Options};

/// Display a banner with the given text using cfonts
pub fn print_banner(text: &str) {
    let mut options = Options::default();
    options.text = text.to_string();
    options.gradient = vec!["#ff0000".into(), "#0000ff".into()];
    let output = render(options);
    println!("{}", output.text);
}
