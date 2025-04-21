mod event_handlers;
mod hinter;

use hinter::Hinter;
use radix_trie::Trie;
use rustyline::{history::DefaultHistory, Cmd, EventHandler, KeyCode, KeyEvent, Modifiers};

use event_handlers::TabEventHandler;
pub use hinter::CommandHint;

pub struct Editor {
    rl: rustyline::Editor<Hinter, DefaultHistory>,
}

impl Editor {
    pub fn new(hints: Vec<CommandHint>) -> rustyline::Result<Self> {
        let mut rl = rustyline::Editor::<Hinter, DefaultHistory>::new()?;

        let hints: Trie<String, CommandHint> = hints
            .into_iter()
            .map(|hint| (hint.display().to_string(), hint))
            .collect();

        let hinter = Hinter::new(hints);
        rl.set_helper(Some(hinter));
        rl.bind_sequence(
            KeyEvent::from('\t'),
            EventHandler::Conditional(Box::new(TabEventHandler)),
        );
        rl.bind_sequence(
            KeyEvent(KeyCode::Enter, Modifiers::SHIFT),
            EventHandler::Simple(Cmd::Newline),
        );

        Ok(Self { rl })
    }

    pub fn readline(&mut self) -> rustyline::Result<String> {
        self.rl.readline("")
    }

    pub fn append_history(&mut self, line: &str) -> rustyline::Result<bool> {
        self.rl.add_history_entry(line)
    }

    pub fn execute_command(&mut self, line: &str) -> Option<String> {
        match self.rl.helper_mut() {
            Some(hinter) => hinter.run_command(line),
            None => Some(line.to_string()),
        }
    }
}
