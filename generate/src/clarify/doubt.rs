use tolearn_core::block::Block;
use tolearn_core::state::Turn;

use crate::stage::Place;

#[derive(Debug, Clone, Copy)]
pub struct Doubt<'a> {
    pub program: &'a str,
    pub node: &'a str,
    pub place: Place<'a>,
    pub block: &'a Block,
    pub chain: &'a [Turn],
    pub question: Option<&'a str>,
}
