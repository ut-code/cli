pub struct ContentEntry {
    pub name: &'static str,
    pub detail: &'static str,
}

pub const CATEGORIES: &[&str] = &["About", "Projects", "Articles", "Members"];

pub const PROJECTS: &[ContentEntry] = &[
    ContentEntry {
        name: "shortcut-game",
        detail: "A game for learning keyboard shortcuts, targeting users who want to improve their speed and efficiency in editors/OS. Built with SvelteKit.",
    },
    ContentEntry {
        name: "my-code",
        detail: "An interactive learning platform for programming, aimed at beginners. Features in-browser code execution and AI chat to help students learn. Built with TypeScript and AI services.",
    },
    ContentEntry {
        name: "cli",
        detail: "A terminal-based CLI made for ut.code(); members to browse info and resources conveniently. Implemented in Rust using ratatui.",
    },
    ContentEntry {
        name: "syllabus",
        detail: "A web app to help university students navigate course registration and class planning. Uses TypeScript and web data scraping for accurate info.",
    },
    ContentEntry {
        name: "space-simulator",
        detail: "An educational web app simulating orbital mechanics and space environments for students interested in physics/space. Implemented with React, TypeScript, and Vite.",
    },
    ContentEntry {
        name: "CourseMate",
        detail: "A web app for students to discover and connect with classmates taking the same university courses. Built with TypeScript and web technologies.",
    },
    ContentEntry {
        name: "create-cpu",
        detail: "A browser-based circuit/CPU simulator aimed at students learning computer architecture. Lets users create logic circuits by connecting gates and wires. Built with TypeScript for the web.",
    },
    ContentEntry {
        name: "dot-turor",
        detail: "A planned app to help new developers set up their dotfiles and dev environments with best practices. Target: beginner programmers. Tech: To be decided.",
    },
    ContentEntry {
        name: "menu",
        detail: "A meal proposal app that suggests menus based on user answers to a series of questions. Developed in Jupyter Notebook and Python.",
    },
    ContentEntry {
        name: "rollcron",
        detail: "A cron job scheduler designed for automated, version-controlled workflows, synchronizing job definitions from git repositories. Implemented in Rust.",
    },
    ContentEntry {
        name: "memomap",
        detail: "A spatial note-taking app for leaving geo-tagged memos on a map. Built as a mobile-first application with Flutter (Dart).",
    },
    ContentEntry {
        name: "discord-bots",
        detail: "A set of Discord bots enabling automation, moderation, and notifications in the ut.code(); community server. Implemented in TypeScript.",
    },
    ContentEntry {
        name: "komabanavi",
        detail: "A web map app for campus navigation. Designed for students and visitors to find buildings and facilities at Komaba campus. Built with TypeScript.",
    },
    ContentEntry {
        name: "hitori-mahjong",
        detail: "A solo play mahjong app aimed at practicing tile recognition and scoring. Built as a web app using TypeScript.",
    },
    ContentEntry {
        name: "itsuhima",
        detail: "A web app for easily coordinating shared availability for small groups and events. Targeted at university students. Written in TypeScript.",
    },
    ContentEntry {
        name: "extralearn",
        detail: "The next step after ut.code(); Learn: an extended learning platform with advanced MDX-based content, especially for those wanting to deepen their programming knowledge. Uses React/TypeScript/MDX.",
    },
    ContentEntry {
        name: "Raxcel",
        detail: "A desktop spreadsheet app for users seeking a lightweight, Excel-style experience on their desktop. Built with Svelte and desktop frameworks.",
    },
    ContentEntry {
        name: "card-game",
        detail: "A web-based card game platform for experimenting with turn-based mechanics and multiplayer support. Built with TypeScript for the web.",
    },
    ContentEntry {
        name: "ut-bridge",
        detail: "A TypeScript app designed to help connect students across university departments and interests. Was used for peer introduction and collaboration. Currently suspended.",
    },
];

pub const MEMBERS: &[ContentEntry] = &[
    ContentEntry {
        name: "nakomochi",
        detail: "Current (5th) representative of ut.code();. Leader of the shortcut-game project and active in web app and CLI tooling across Svelte, TypeScript, and Rust.",
    },
    ContentEntry {
        name: "chvmvd",
        detail: "3rd representative of ut.code();. Key maintainer of educational materials (including utcode-learn), focused on algorithm education and open-source TypeScript projects.",
    },
    ContentEntry {
        name: "aster-void",
        detail: "Maintainer of extralearn (extra.utcode.net). Focused on \
                 systems programming and low-level tooling.",
    },
    ContentEntry {
        name: "na-trium-144",
        detail: "Leader of the my-code project. Prolific contributor spanning \
                 TypeScript, C++, and systems tooling. Creator of webcface \
                 (a web-based communication framework) and Nikochan.",
    },
    ContentEntry {
        name: "nakaterm",
        detail: "Interested in terminal UIs and developer experience. Author \
                 of multiple open-source CLI utilities within the ut.code(); \
                 ecosystem.",
    },
    ContentEntry {
        name: "Tsurubara-UTcode",
        detail: "Interested in computer vision and hackathon projects. \
                 Contributed to team projects including umaproject and \
                 coetecohack.",
    },
    ContentEntry {
        name: "tknkaa",
        detail: "Builder of this CLI. Interested in Rust, TUI design, and \
                 developer tooling. Maintains hitori-mahjong and Raxcel.",
    },
    ContentEntry {
        name: "chelproc",
        detail: "Founder and 1st representative of ut.code();. TypeScript developer, contributed setup scripts and infrastructure for community learning servers.",
    },
    ContentEntry {
        name: "Tatsu723",
        detail: "An ut.code(); member actively contributing to community \
                 development projects.",
    },
    ContentEntry {
        name: "faithia-anastasia",
        detail: "An ut.code(); member diving into the community and its \
                 projects.",
    },
    ContentEntry {
        name: "shirokuma222",
        detail: "An ut.code(); member contributing to community projects \
                 and development initiatives.",
    },
    ContentEntry {
        name: "RRRyoma",
        detail: "An ut.code(); member contributing to community projects \
                 and development initiatives.",
    },
    ContentEntry {
        name: "Yokomi422",
        detail: "An ut.code(); member engaged in community projects and \
                 collaborative development.",
    },
    ContentEntry {
        name: "tsatohiro",
        detail: "An ut.code(); member contributing to community projects \
                 and development initiatives.",
    },
    ContentEntry {
        name: "SotaTamura",
        detail: "An ut.code(); member contributing to the community's \
                 growing portfolio of projects.",
    },
    ContentEntry {
        name: "Rn86222",
        detail: "An ut.code(); member bringing research depth and academic \
                 perspective to community projects.",
    },
    ContentEntry {
        name: "taka231",
        detail: "Interested in systems programming and compilers. Built an \
                 HLS compiler, eBPF loader, WebAssembly JIT, Brainfuck JIT, \
                 and a C compiler (recc), mostly in Rust.",
    },
    ContentEntry {
        name: "KaichiManabe",
        detail: "The 4th representative of ut.code();, taking office from \
                 September 2025. Leads the community's direction and \
                 coordinates projects and events.",
    },
    ContentEntry {
        name: "brdgb",
        detail: "Interested in bridging systems engineering and software \
                 development. Contributes an interdisciplinary perspective \
                 to ut.code(); projects.",
    },
    ContentEntry {
        name: "claude",
        detail: "An AI assistant by Anthropic and honorary member of the \
                 ut.code(); community. Helps members with code, documentation, \
                 and this very CLI.",
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
