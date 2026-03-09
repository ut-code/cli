pub struct ContentEntry {
    pub name: &'static str,
    pub detail: &'static str,
}

pub const CATEGORIES: &[&str] = &["Projects", "Members"];

pub const PROJECTS: &[ContentEntry] = &[
    ContentEntry {
        name: "shortcut-game",
        detail: "A keyboard shortcut learning game built with SvelteKit. \
                 Players practice common editor and OS shortcuts in an \
                 interactive format, available at shortcut-game.utcode.net.",
    },
    ContentEntry {
        name: "my-code",
        detail: "An AI-powered interactive web learning platform for \
                 programming, live at my-code.utcode.net. Features in-browser \
                 REPLs, code editors, and an AI chat assistant via Gemini.",
    },
    ContentEntry {
        name: "cli",
        detail: "A command-line interface tool for the ut.code(); community. \
                 Provides quick access to member info, projects, and community \
                 resources directly from the terminal.",
    },
    ContentEntry {
        name: "syllabus",
        detail: "A course registration support system nicknamed Shiraku Bus. \
                 Helps University of Tokyo students navigate course enrollment \
                 with TypeScript-based tooling and scraped course data.",
    },
    ContentEntry {
        name: "space-simulator",
        detail: "An early-stage space simulator web app built with React, \
                 TypeScript, and Vite. Simulates orbital mechanics and \
                 space environments, hosted at space-simulator.utcode.net.",
    },
    ContentEntry {
        name: "CourseMate",
        detail: "A course management and peer-matching platform built by \
                 ut.code(); members. Helps students find study partners and \
                 organize collaborative learning sessions.",
    },
    ContentEntry {
        name: "create-cpu",
        detail: "A browser-based interactive CPU and circuit simulator. \
                 Users build circuits from logic gates, connect pins and wires, \
                 and compose arbitrary logic to understand computer architecture.",
    },
];

pub const MEMBERS: &[ContentEntry] = &[
    ContentEntry {
        name: "nakomochi",
        detail: "Leader of the shortcut-game project and active ut.code(); \
                 member. Works across Svelte, TypeScript, and Rust, with \
                 interests spanning web apps and CLI tooling.",
    },
    ContentEntry {
        name: "chvmvd",
        detail: "A GitHub Pro contributor and key maintainer of ut.code(); \
                 educational materials, including utcode-learn. Active in \
                 algorithm education and open-source TypeScript projects.",
    },
    ContentEntry {
        name: "aster-void",
        detail: "A core contributor focused on systems programming and \
                 low-level tooling. Maintains several foundational libraries \
                 used across ut.code(); projects.",
    },
    ContentEntry {
        name: "na-trium-144",
        detail: "Leader of the my-code project and prolific contributor \
                 spanning TypeScript, C++, and systems tooling. Creator of \
                 webcface (a web-based communication framework) and Nikochan.",
    },
    ContentEntry {
        name: "nakaterm",
        detail: "A full-stack developer with a passion for terminal UIs and \
                 developer experience. Author of multiple open-source CLI \
                 utilities within the ut.code(); ecosystem.",
    },
    ContentEntry {
        name: "Tsurubara-UTcode",
        detail: "An ut.code(); member with experience in hackathon projects \
                 and computer vision work. Contributed to team projects \
                 including umaproject and coetecohack.",
    },
    ContentEntry {
        name: "tknkaa",
        detail: "Builder of this CLI and active ut.code(); member. Interested \
                 in Rust, TUI design, and making developer tools that are both \
                 fast and pleasant to use.",
    },
    ContentEntry {
        name: "chelproc",
        detail: "An ut.code(); member and TypeScript developer. Contributed \
                 to community tooling including setup scripts for ut.code(); \
                 learning servers and various open-source projects.",
    },
];

pub fn content_entries(category: usize) -> &'static [ContentEntry] {
    match category {
        0 => PROJECTS,
        1 => MEMBERS,
        _ => &[],
    }
}
