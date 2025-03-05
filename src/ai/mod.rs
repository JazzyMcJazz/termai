mod ask;
mod chat;
mod explain;
mod suggest;

use ask::ask;
use chat::chat;
use console::Term;
use explain::explain;
use suggest::suggest;

use crate::config::Config;

pub struct AI<'a> {
    term: &'a Term,
    cfg: &'a Config,
}

impl AI<'_> {
    pub fn new<'a>(term: &'a Term, cfg: &'a Config) -> AI<'a> {
        AI::<'a> { term, cfg }
    }

    pub fn chat(&self, initial_message: Option<String>) {
        chat(self.term, self.cfg, initial_message);
    }

    pub fn explain(&self, query: Option<String>) {
        explain(self.cfg, query);
    }

    pub fn suggest(&self, query: Option<String>) {
        suggest(self.cfg, query);
    }

    pub fn ask(&self, message: Option<String>) {
        ask(self.term, self.cfg, message);
    }
}
