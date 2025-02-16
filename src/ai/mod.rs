mod chat;
mod explain;
mod suggest;

use chat::chat;
use console::Term;
use explain::explain;
use suggest::suggest;

use crate::provider::Provider;

pub struct AI<'a>(pub &'a Term);

impl AI<'_> {
    pub fn chat(&self, provider: &Provider, initial_message: Option<String>, streaming: bool) {
        chat(self.0, provider, initial_message, streaming);
    }

    pub fn explain(&self, provider: &Provider, query: Option<String>) {
        explain(provider, query);
    }

    pub fn suggest(&self, provider: &Provider, query: Option<String>) {
        suggest(provider, query);
    }
}
