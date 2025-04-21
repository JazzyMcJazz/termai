use rustyline::{Cmd, ConditionalEventHandler, Event, EventContext, KeyEvent, RepeatCount};

pub struct TabEventHandler;

impl ConditionalEventHandler for TabEventHandler {
    fn handle(&self, evt: &Event, n: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {
        debug_assert_eq!(*evt, Event::from(KeyEvent::from('\t')));
        ctx.hint_text().map(|hint| Cmd::Insert(n, hint.to_owned()))
    }
}
