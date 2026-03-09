pub struct ContentEntry {
    pub name: &'static str,
    pub detail: &'static str,
}

pub const CATEGORIES: &[&str] = &["About", "Projects", "Articles", "Members"];

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

pub const ARTICLES: &[ContentEntry] = &[
    ContentEntry {
        name: "Hackathon (2025/12/21)",
        detail: "ut.code(); held an internal hackathon in December themed \
                 \"CLI tool\" with a tech restriction of Python or Rust. \
                 Teams kicked off on Dec 6, built projects over several days, \
                 and presented results. A fun challenge outside the usual TypeScript stack.",
    },
    ContentEntry {
        name: "6th General Meeting (2025/12/13)",
        detail: "The 6th ut.code(); general assembly was held on December 13. \
                 The meeting included a leadership handover ceremony and updates \
                 from active projects on their progress.",
    },
    ContentEntry {
        name: "5th General Meeting (2025/11/29)",
        detail: "The 5th ut.code(); general assembly was held on November 29. \
                 Standing projects presented their annual work, and the community \
                 reviewed ongoing initiatives.",
    },
    ContentEntry {
        name: "Komaba Festival (2025/11/28)",
        detail: "ut.code(); exhibited at the 76th Komaba Festival (Nov 22-24, 2025) \
                 with a booth called \"Programming Just for You\". Members demonstrated \
                 projects and engaged visitors with interactive programming experiences.",
    },
    ContentEntry {
        name: "4th General Meeting (2025/11/12)",
        detail: "The 4th ut.code(); general assembly was held on October 25. \
                 Standing projects reported progress, and the community discussed \
                 upcoming events and initiatives.",
    },
    ContentEntry {
        name: "Komaba Fest Kickoff (2025/10/11)",
        detail: "ut.code(); held a kickoff meeting on Oct 11 to prepare for the \
                 76th Komaba Festival. Teams selected projects to exhibit and \
                 planned their interactive demonstrations.",
    },
    ContentEntry {
        name: "RIZAP Hackathon (2025/09/30)",
        detail: "ut.code(); co-hosted a hackathon with RIZAP Technologies on Sep 30 \
                 under the theme \"Health x Technology\". Teams built products \
                 combining wellness concepts with software engineering.",
    },
    ContentEntry {
        name: "3rd General Meeting (2025/09/27)",
        detail: "The 3rd ut.code(); general assembly marked the first meeting under \
                 the new leadership. Projects presented updates and the community \
                 discussed its direction for the rest of the year.",
    },
    ContentEntry {
        name: "New Leadership (2025/09/23)",
        detail: "ut.code(); announced a leadership change effective September 1, 2025. \
                 Kaichi Manabe became the 4th representative, taking over from the \
                 previous leader.",
    },
    ContentEntry {
        name: "Summer Camp (2025/09/17)",
        detail: "ut.code(); held its summer retreat Sep 17-19 at the University of \
                 Tokyo Yamanaka Seminar House. Members worked on projects, held \
                 technical sessions, and bonded as a community.",
    },
    ContentEntry {
        name: "GMO Office Tour (2025/08/22)",
        detail: "First and second-year ut.code(); members visited GMO Media's office \
                 on Aug 22 for an office tour and engineer roundtable, gaining insight \
                 into professional software development.",
    },
    ContentEntry {
        name: "May Festival (2025/06/08)",
        detail: "ut.code(); exhibited at the 98th May Festival (May 24-25, 2025). \
                 Displays included intro programming workshops, a hackathon showcase \
                 by new members, and the shortcut-key puzzle game.",
    },
    ContentEntry {
        name: "New Member Camp (2025/05/12)",
        detail: "ut.code(); held its first-ever new member retreat May 3-5 at \
                 the Yamanaka Seminar House. New members bonded and got oriented \
                 to the community during Golden Week.",
    },
    ContentEntry {
        name: "Newcomers Hackathon (2025/05/11)",
        detail: "ut.code(); ran a two-day newcomer hackathon May 10-11, giving new \
                 members hands-on experience building projects from scratch and \
                 presenting their work to the community.",
    },
    ContentEntry {
        name: "Joint Welcome Event (2025/03/24)",
        detail: "ut.code(); co-organized a joint welcome event for incoming students \
                 alongside other University of Tokyo engineering clubs, held on \
                 Apr 9 and Apr 15.",
    },
    ContentEntry {
        name: "Spring Camp (2025/03/01)",
        detail: "ut.code(); held its spring retreat Feb 20-22 at the Yamanaka \
                 Seminar House. Members visited Lake Yamanaka, worked on projects, \
                 and participated in team activities.",
    },
    ContentEntry {
        name: "2nd General Meeting (2025/02/16)",
        detail: "The 2nd ut.code(); general assembly focused on project progress \
                 reports and recent technology topics. The community established \
                 a cadence of meetings every 1-2 months.",
    },
    ContentEntry {
        name: "Project Launch Party (2024/12/22)",
        detail: "ut.code(); held a project founding party on Dec 21, 2024, where \
                 members brainstormed product ideas over pizza and voted on new \
                 projects to pursue in 2025.",
    },
];

pub const ABOUT: &[ContentEntry] = &[
    ContentEntry {
        name: "Overview",
        detail: "ut.code(); is a software engineering community at the University \
                 of Tokyo, founded in 2019. Around 30 members strong, the club \
                 is based in room 313B of the Komaba Student Hall and affiliated \
                 with the Faculty of Engineering Teiyukai (2025).",
    },
    ContentEntry {
        name: "Activities",
        detail: "The club operates across three pillars:\n\
                 - Learning & Education: ut.code(); Learn, seminars\n\
                 - Exchange: May Festival, Komaba Festival, retreats\n\
                 - Development: projects, hackathons\n\n\
                 Members typically meet weekly for evening project sessions \
                 plus 1-2 work sessions per month.",
    },
    ContentEntry {
        name: "Beginners Welcome",
        detail: "Complete beginners are fully welcome. ut.code(); publishes \
                 learning materials that take total novices to building full-stack \
                 applications, backed by study groups and workshops. No restrictions \
                 on university, year, or academic background.",
    },
    ContentEntry {
        name: "Tech Stack",
        detail: "Specific tech varies by project, but common choices include:\n\
                 - Language: TypeScript, Rust, Python\n\
                 - Frontend: React / Next.js, Svelte(Kit)\n\
                 - UI: MUI, Tailwind, DaisyUI\n\
                 - Backend: Hono, Express, Prisma, Drizzle\n\
                 - DB: Supabase, Neon\n\
                 - Infra: Cloudflare, Fly.io, Render\n\
                 - Tools: Notion, Discord, GitHub",
    },
    ContentEntry {
        name: "Locations",
        detail: "ut.code(); members work from multiple locations:\n\
                 - Club room: Komaba Student Hall 313B\n\
                 - Hongo Library Project Box\n\
                 - Online via Discord",
    },
    ContentEntry {
        name: "Social & Contact",
        detail: "Find ut.code(); online:\n\
                 - X (Twitter): @utokyo_code\n\
                 - GitHub: github.com/ut-code\n\
                 - Website: utcode.net\n\n\
                 Donations and sponsorships are welcome — see utcode.net/donation for details.",
    },
];

pub fn content_entries(category: usize) -> &'static [ContentEntry] {
    match category {
        0 => ABOUT,
        1 => PROJECTS,
        2 => ARTICLES,
        3 => MEMBERS,
        _ => &[],
    }
}
