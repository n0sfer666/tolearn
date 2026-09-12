use std::sync::Arc;

use tolearn_generate::ledger::Tally;

#[derive(Debug, Clone, Default)]
pub struct Ledger(Arc<Tally>);

impl Ledger {
    pub fn tally(&self) -> &Tally {
        &self.0
    }
}
