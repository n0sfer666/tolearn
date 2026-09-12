use std::collections::BTreeSet;

use tolearn_core::Hours;
use tolearn_core::program::{MAX_DEPTH, Map};
use tolearn_core::yaml::is_slug;

use super::flaw::Flaw;
use super::types::Plan;
use super::{MAX_HOURS, MAX_STAGES, stage_fits};

pub fn check(plan: &Plan, depth: usize) -> Vec<Flaw> {
    let mut flaws = Vec::new();
    blank(&plan.title, "название программы", &mut flaws);
    blank(&plan.goal, "цель программы", &mut flaws);
    if !is_slug(&plan.slug) {
        flaws.push(Flaw::Slug(plan.slug.clone()));
    }
    match (plan.stages.is_empty(), plan.children.is_empty()) {
        (true, true) => flaws.push(Flaw::Empty),
        (false, false) => flaws.push(Flaw::Mixed),
        (false, true) => stages(plan, &mut flaws),
        (true, false) => children(plan, depth, &mut flaws),
    }
    flaws
}

fn stages(plan: &Plan, flaws: &mut Vec<Flaw>) {
    let mut seen = BTreeSet::new();
    for row in &plan.stages {
        blank(&row.title, &format!("название этапа «{}»", row.id), flaws);
        if !is_slug(&row.id) {
            flaws.push(Flaw::StageId(row.id.clone()));
        } else if !seen.insert(row.id.as_str()) {
            flaws.push(Flaw::DuplicateId(row.id.clone()));
        }
        if !stage_fits(row.hours) {
            flaws.push(Flaw::StageHours {
                id: row.id.clone(),
                hours: row.hours,
            });
        }
    }
    if plan.stages.len() > MAX_STAGES {
        flaws.push(Flaw::LeafStages(plan.stages.len()));
    }
    let total = Map {
        stages: plan.stages.clone(),
        children: Vec::new(),
    }
    .hours();
    if total.max > MAX_HOURS {
        flaws.push(Flaw::LeafHours(total));
    }
}

fn children(plan: &Plan, depth: usize, flaws: &mut Vec<Flaw>) {
    if depth >= MAX_DEPTH {
        flaws.push(Flaw::TooDeep { depth });
        return;
    }
    for (number, row) in plan.children.iter().enumerate() {
        blank(
            &row.title,
            &format!("название подпрограммы №{}", number + 1),
            flaws,
        );
        blank(
            &row.goal,
            &format!("цель подпрограммы «{}»", row.title),
            flaws,
        );
        let Hours { min, max } = row.hours;
        if min == 0 || max > MAX_HOURS || min > max {
            flaws.push(Flaw::PartHours {
                title: row.title.clone(),
                hours: row.hours,
            });
        }
    }
}

fn blank(text: &str, what: &str, flaws: &mut Vec<Flaw>) {
    if text.trim().is_empty() {
        flaws.push(Flaw::Blank(what.to_owned()));
    }
}
