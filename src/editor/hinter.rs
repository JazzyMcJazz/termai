use std::borrow::Cow::{self, Borrowed, Owned};

use derivative::Derivative;
use radix_trie::{Trie, TrieCommon};
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hint, Hinter as ReadlineHinter};
use rustyline::Context;
use rustyline::{Completer, Helper, Validator};

#[derive(Completer, Helper, Validator)]
pub struct Hinter {
    hints: Trie<String, CommandHint>,
}

impl Hinter {
    pub fn new(hints: Trie<String, CommandHint>) -> Self {
        Self { hints }
    }

    pub fn run_command(&mut self, line: &str) -> Option<String> {
        // Check if any prefix of the line exists in the hints trie
        // We want an exact match of a key as a prefix of the line
        let mut prefix = line;
        while !prefix.is_empty() {
            if let Some(hint) = self.hints.get_mut(prefix) {
                return (hint.command)(line);
            }
            prefix = &prefix[..prefix.len() - 1];
        }
        Some(line.to_string())
    }
}

impl ReadlineHinter for Hinter {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<CommandHint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        let prefix = line;
        self.hints.get_raw_descendant(prefix).and_then(|node| {
            if let Some(hint) = node.value() {
                Some(hint.suffix(pos))
            } else {
                // If the node doesn't have a value, find the first child with a value
                node.iter().map(|(_, hint)| hint.suffix(pos)).next()
            }
        })
    }
}

impl Highlighter for Hinter {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, _: bool) -> Cow<'b, str> {
        Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("\x1b[2m{hint}\x1b[m"))
    }
}

type CommandFn = Box<dyn Fn(&str) -> Option<String>>;

#[derive(Derivative)]
#[derivative(Hash, Debug, PartialEq, Eq)]
pub struct CommandHint {
    display: String,
    complete_up_to: usize,
    #[derivative(Hash = "ignore", Debug = "ignore", PartialEq = "ignore")]
    command: CommandFn,
}

impl Hint for CommandHint {
    fn display(&self) -> &str {
        &self.display
    }

    fn completion(&self) -> Option<&str> {
        if self.complete_up_to > 0 {
            Some(&self.display[..self.complete_up_to])
        } else {
            None
        }
    }
}

impl CommandHint {
    pub fn new(text: &str, complete_up_to: &str, command: CommandFn) -> Self {
        debug_assert!(text.starts_with(complete_up_to));
        Self {
            display: text.to_owned(),
            complete_up_to: complete_up_to.len(),
            command,
        }
    }

    pub fn display(&self) -> &str {
        &self.display
    }

    fn suffix(&self, strip_chars: usize) -> Self {
        let hint = self.display[strip_chars..].to_owned();
        Self {
            display: hint.clone(),
            complete_up_to: self.complete_up_to.saturating_sub(strip_chars),
            command: Box::new(|_| None),
        }
    }
}
