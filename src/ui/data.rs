pub struct ContentEntry {
    pub name: &'static str,
    pub detail: &'static str,
}

pub const CATEGORIES: &[&str] = &["Projects", "Members"];

pub const PROJECTS: &[ContentEntry] = &[
    ContentEntry {
        name: "ut.code(); CLI",
        detail: "A command-line interface tool for the ut.code(); community. \
                 Provides quick access to member info, projects, and community \
                 resources directly from the terminal.",
    },
    ContentEntry {
        name: "coursemate",
        detail: "A course management and peer-matching platform built by \
                 ut.code(); members. Helps students find study partners and \
                 organize collaborative learning sessions.",
    },
    ContentEntry {
        name: "ut.code(); Learn",
        detail: "An interactive learning platform tailored for the ut.code(); \
                 curriculum. Offers structured lessons, quizzes, and progress \
                 tracking for members at all skill levels.",
    },
];

pub const MEMBERS: &[ContentEntry] = &[
    ContentEntry {
        name: "aster-void",
        detail: "A core contributor focused on systems programming and \
                 low-level tooling. Maintains several foundational libraries \
                 used across ut.code(); projects.",
    },
    ContentEntry {
        name: "nakaterm",
        detail: "A full-stack developer with a passion for terminal UIs and \
                 developer experience. Author of multiple open-source CLI \
                 utilities within the ut.code(); ecosystem.",
    },
    ContentEntry {
        name: "tknkaa",
        detail: "Builder of this CLI and active ut.code(); member. Interested \
                 in Rust, TUI design, and making developer tools that are both \
                 fast and pleasant to use.",
    },
];

pub fn content_entries(category: usize) -> &'static [ContentEntry] {
    match category {
        0 => PROJECTS,
        1 => MEMBERS,
        _ => &[],
    }
}
