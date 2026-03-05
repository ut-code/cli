use cfonts::{render, say, Colors, Fonts, Options, Rgb};

pub fn run() {
    // "We're": c1=white (body), c2=#00ff87 (connectors) — matches original colors={["#ffffff","#00ff87"]}
    say(Options {
        text: String::from("We're"),
        font: Fonts::FontBlock,
        colors: vec![Colors::White, Colors::Rgb(Rgb::Val(0, 255, 135))],
        spaceless: true,
        ..Options::default()
    });

    // "ut." (system) + "code" (green) + "();" (system) rendered side-by-side,
    // mirroring the original's <Box> with three adjacent <BigText> components.
    let parts: &[(&str, Vec<Colors>)] = &[
        ("ut.", vec![Colors::System]),
        ("code", vec![Colors::Green]),
        ("();", vec![Colors::System]),
    ];

    let row_groups: Vec<Vec<String>> = parts
        .iter()
        .map(|(text, colors)| {
            render(Options {
                text: String::from(*text),
                font: Fonts::FontBlock,
                colors: colors.clone(),
                spaceless: true,
                ..Options::default()
            })
            .text
            .lines()
            .map(String::from)
            .collect()
        })
        .collect();

    let num_rows = row_groups.iter().map(|g| g.len()).max().unwrap_or(0);
    for i in 0..num_rows {
        let line: String = row_groups
            .iter()
            .map(|g| g.get(i).map(String::as_str).unwrap_or(""))
            .collect();
        println!("{line}");
    }
}
