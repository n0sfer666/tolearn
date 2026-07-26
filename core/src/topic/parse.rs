use super::enums::{
    CONFIDENCE, LIVENESS, MATERIAL_TIER, MATERIAL_TYPE, PRACTICE_KIND, PRACTICE_TIER,
    QUESTION_TYPE, RETENTION, VOLATILITY,
};
use super::types::{Check, Exam, Material, Practice, Question, Topic};
use crate::hours::hours;
use crate::yaml::{ParseError, Reader, read};

pub fn parse(source: &str) -> Result<Topic, ParseError> {
    read(source, topic)
}

fn topic(node: &Reader<'_>) -> Result<Topic, ParseError> {
    Ok(Topic {
        schema: node.field("schema")?.text()?,
        id: node.field("id")?.text()?,
        title: node.field("title")?.text()?,
        stage: node.field("stage")?.number(1)?,
        depends_on: node.field("depends_on")?.texts()?,
        est_hours: hours(&node.field("est_hours")?)?,
        volatility: node
            .field("volatility")?
            .choice("volatility", &VOLATILITY)?,
        revalidate_after_days: node.field("revalidate_after_days")?.number(1)?,
        verified_at: node.field("verified_at")?.text()?,
        confidence: node
            .field("confidence")?
            .choice("confidence", &CONFIDENCE)?,
        retention: node.field("retention")?.choice("retention", &RETENTION)?,
        version_context: node.field("version_context")?.texts()?,
        outcomes: node.field("outcomes")?.texts()?,
        misconceptions: node.field("misconceptions")?.texts()?,
        materials: node.field("materials")?.list(material)?,
        practice: practice(&node.field("practice")?)?,
        questions: node.field("questions")?.list(question)?,
        exam: exam(&node.field("exam")?)?,
    })
}

fn material(node: &Reader<'_>) -> Result<Material, ParseError> {
    Ok(Material {
        title: node.field("title")?.text()?,
        url: node.field("url")?.text()?,
        kind: node
            .field("type")?
            .choice("material type", &MATERIAL_TYPE)?,
        tier: node
            .field("tier")?
            .choice("material tier", &MATERIAL_TIER)?,
        lang: node.field("lang")?.text()?,
        liveness: node.field("liveness")?.choice("liveness", &LIVENESS)?,
        published: node.field("published")?.optional_text()?,
        covers_version: node.field("covers_version")?.optional_text()?,
        checked_at: node.field("checked_at")?.text()?,
        stale: node.field("stale")?.flag()?,
        delta: node.field("delta")?.optional_text()?,
        note: node.field("note")?.text()?,
    })
}

fn practice(node: &Reader<'_>) -> Result<Practice, ParseError> {
    Ok(Practice {
        kind: node
            .field("kind")?
            .choice("practice kind", &PRACTICE_KIND)?,
        tier: node
            .field("tier")?
            .choice("practice tier", &PRACTICE_TIER)?,
        task: node.field("task")?.text()?,
        deliverable: node.field("deliverable")?.text()?,
        starting_point: node.field("starting_point")?.optional_text()?,
        fallback: node.field("fallback")?.optional_text()?,
        time_box_min: node.field("time_box_min")?.number(1)?,
        smoke_checked: node.field("smoke_checked")?.flag()?,
        constraints: node.field("constraints")?.list(check)?,
        acceptance: node.field("acceptance")?.list(check)?,
    })
}

fn check(node: &Reader<'_>) -> Result<Check, ParseError> {
    Ok(Check {
        id: node.field("id")?.text()?,
        claim: node.field("claim")?.text()?,
        check: node.field("check")?.text()?,
        expect: node.field("expect")?.text()?,
    })
}

fn question(node: &Reader<'_>) -> Result<Question, ParseError> {
    Ok(Question {
        id: node.field("id")?.text()?,
        kind: node
            .field("type")?
            .choice("question type", &QUESTION_TYPE)?,
        text: node.field("text")?.text()?,
        expected_signals: node.field("expected_signals")?.texts()?,
        red_flags: node.field("red_flags")?.texts()?,
        follow_up: node.field("follow_up")?.optional_text()?,
    })
}

fn exam(node: &Reader<'_>) -> Result<Exam, ParseError> {
    Ok(Exam {
        focus: node.field("focus")?.text()?,
        traps: node.field("traps")?.texts()?,
        artifact_required: node.field("artifact_required")?.flag()?,
        max_exchanges: node.field("max_exchanges")?.number(1)?,
    })
}
