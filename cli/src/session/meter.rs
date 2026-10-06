use std::cell::Cell;

use tolearn_provider::Tokens;

use super::spent::Spent;

#[derive(Debug, Default)]
pub struct Meter(Cell<Spent>);

impl Meter {
    pub fn count(&self, tokens: Tokens) {
        let mut spent = self.0.get();
        spent.calls = spent.calls.saturating_add(1);
        spent.input = spent
            .input
            .saturating_add(u64::from(tokens.input.unwrap_or(0)));
        spent.output = spent
            .output
            .saturating_add(u64::from(tokens.output.unwrap_or(0)));
        if tokens.input.is_none() || tokens.output.is_none() {
            spent.unknown = spent.unknown.saturating_add(1);
        }
        self.0.set(spent);
    }

    pub fn spent(&self) -> Spent {
        self.0.get()
    }
}
