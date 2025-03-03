use console::Term;
use indicatif::TermLike;
use termimad::MadSkin;

static CHANGELOG: &str = include_str!("../../CHANGELOG.md");
static SEE_MORE: &str =
    "**See more changelogs at**\nhttps://github.com/JazzyMcJazz/termai/blob/main/CHANGELOG.md";

pub fn print_latest() {
    let term = Term::stdout();

    let sections = CHANGELOG.split("\n## ").collect::<Vec<&str>>();

    let latest = if CHANGELOG.contains("## [Unreleased]") {
        sections.get(2)
    } else {
        sections.get(1)
    };

    if let Some(latest) = latest {
        let skin = MadSkin::default();
        let text = format!("# CHANGELOG\n\n**Version {}**\n{}", latest, SEE_MORE);
        let text = skin.text(&text, Some(term.width() as usize));

        println!("\n{}", text);
    } else {
        println!("\nNo changelog found.");
    }
}
