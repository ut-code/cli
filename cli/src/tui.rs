use cfonts::{render, Colors, Options, Rgb};

/// Display a banner with the given text using cfonts
pub fn print_banner(text: &str) {
    let mut options = Options::default();
    options.colors = vec![
        Colors::Rgb(Rgb::Val(72, 214, 108)),
        Colors::Rgb(Rgb::Val(72, 214, 108)),
    ];
    options.text = text.to_string();
    options.letter_spacing = 0;
    let output = render(options);
    println!("{}", output.text);
}
