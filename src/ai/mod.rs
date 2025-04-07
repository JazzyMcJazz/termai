mod chat;
mod explain;
mod suggest;

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

    pub fn chat(&self, initial_message: Option<String>, _model: bool, _search: bool) {
        chat(self.term, self.cfg, initial_message);
    }

    pub fn explain(&self, query: Option<String>, _model: bool) {
        explain(self.cfg, query);
    }

    pub fn suggest(&self, query: Option<String>, _model: bool) {
        suggest(self.cfg, query);
    }
}
