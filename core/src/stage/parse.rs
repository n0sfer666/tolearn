use super::types::{Check, Practice, Question, Stage};
use crate::block::block;
use crate::yaml::{ParseError, Reader, read};

pub(super) const SCHEMA: &str = "tolearn/stage/1";

pub fn parse(source: &str) -> Result<Stage, ParseError> {
    read(source, stage)
}

fn stage(node: &Reader<'_>) -> Result<Stage, ParseError> {
    node.field("schema")?
        .choice("stage schema", &[(SCHEMA, ())])?;
    Ok(Stage {
        id: node.field("id")?.slug()?,
        title: node.field("title")?.text()?,
        blocks: node.field("blocks")?.list(block)?,
        practice: practice(&node.field("practice")?)?,
        questions: node.field("questions")?.list(question)?,
    })
}

fn practice(node: &Reader<'_>) -> Result<Practice, ParseError> {
    Ok(Practice {
        task: node.field("task")?.list(block)?,
        deliverable: node.field("deliverable")?.text()?,
        constraints: node.field("constraints")?.list(check)?,
        acceptance: node.field("acceptance")?.list(check)?,
    })
}

fn check(node: &Reader<'_>) -> Result<Check, ParseError> {
    Ok(Check {
        id: node.field("id")?.text()?,
        claim: node.field("claim")?.text()?,
        check: node
            .optional_field("check")?
            .as_ref()
            .map(Reader::text)
            .transpose()?,
        expect: node.field("expect")?.text()?,
    })
}

fn question(node: &Reader<'_>) -> Result<Question, ParseError> {
    Ok(Question {
        id: node.field("id")?.text()?,
        text: node.field("text")?.text()?,
        answer: node.field("answer")?.text()?,
    })
}
