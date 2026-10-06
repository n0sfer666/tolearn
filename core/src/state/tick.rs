use super::key::key;
use super::types::State;

impl State {
    pub fn ticks(&self, node: &str, stage: &str) -> Vec<String> {
        self.stages
            .get(&key(node, stage))
            .map(|entry| entry.ticks.clone())
            .unwrap_or_default()
    }

    pub fn tick(&mut self, node: &str, stage: &str, claim: &str, on: bool) -> bool {
        let entry = self.stages.entry(key(node, stage)).or_default();
        let had = entry.ticks.iter().any(|tick| tick == claim);
        if on && !had {
            entry.ticks.push(claim.to_owned());
        }
        if !on {
            entry.ticks.retain(|tick| tick != claim);
        }
        had != on
    }
}
