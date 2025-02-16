use termimad::MadSkin;

static CHANGELOG: &str = include_str!("../../CHANGELOG.md");
static SEE_MORE: &str =
    "**See more changelogs at**\nhttps://github.com/JazzyMcJazz/termai/blob/main/CHANGELOG.md";

pub fn print_latest() {
    let sections = CHANGELOG.split("\n## ").collect::<Vec<&str>>();

    let latest = if CHANGELOG.contains("## [Unreleased]") {
        sections.get(2)
    } else {
        sections.get(1)
    };

    if let Some(latest) = latest {
        let skin = MadSkin::default();
        let text = format!("# CHANGELOG\n\n## {}\n{}", latest, SEE_MORE);
        let longest_line = text
            .lines()
            .max_by_key(|line| line.len())
            .unwrap_or_default()
            .len();
        let text = skin.text(&text, Some(longest_line));

        println!("\n{}", text);
    } else {
        println!("\nNo changelog found.");
    }
}
