use tolearn_core::Hours;
use tolearn_core::program::{StageRow, Volatility};
use tolearn_generate::plan::{Part, Plan, Request};
use tolearn_generate::{GenerateError, Online, online};
use tolearn_provider::Provider;

use super::context::Context;
use super::error::IpcError;
use super::planned::{PlanOut, PlanPartView, PlanStageView, PlanView};
use super::provider::{denied, failed};
use super::settings::stored;
use super::types::Span;
use super::voiced::Voiced;
use crate::journal::Journal;

pub fn drawn(
    context: &Context,
    kind: &'static str,
    request: &str,
    level: &str,
    draw: impl FnOnce(&Online<'_>, &Request) -> Result<Plan, GenerateError>,
) -> Result<PlanOut, IpcError> {
    let request = Request {
        request: told("запрос", request)?,
        level: level.trim().to_owned(),
        locale: stored(context)?.locale.label().to_owned(),
    };
    let provider = Provider::read(&context.provider()).map_err(failed)?;
    let key = context.vault().key().map_err(denied)?;
    let journal = Journal::new(context.llm_log(), provider.journal);
    let model = Voiced::new(provider, key, journal, kind);
    let reach = context.reach()?;
    let online = online(reach.as_ref(), &model).map_err(refused)?;
    let plan = draw(&online, &request).map_err(refused)?;
    Ok(PlanOut {
        hours: span(plan.hours()),
        plan: view(&plan),
    })
}

pub fn told(what: &str, value: &str) -> Result<String, IpcError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(IpcError::new("plan.empty", format!("пусто: {what}")));
    }
    Ok(value.to_owned())
}

pub fn taken(view: &PlanView) -> Result<Plan, IpcError> {
    let volatility = Volatility::parse(&view.volatility).ok_or_else(|| {
        IpcError::new(
            "plan.unknown-value",
            format!("`{}` — не значение для «volatility»", view.volatility),
        )
    })?;
    Ok(Plan {
        title: view.title.clone(),
        slug: view.slug.clone(),
        goal: view.goal.clone(),
        volatility,
        stages: view
            .stages
            .iter()
            .map(|row| StageRow {
                id: row.id.clone(),
                title: row.title.clone(),
                hours: hours(&row.hours),
            })
            .collect(),
        children: view
            .children
            .iter()
            .map(|row| Part {
                title: row.title.clone(),
                goal: row.goal.clone(),
                hours: hours(&row.hours),
            })
            .collect(),
    })
}

fn view(plan: &Plan) -> PlanView {
    PlanView {
        title: plan.title.clone(),
        slug: plan.slug.clone(),
        goal: plan.goal.clone(),
        volatility: plan.volatility.label().to_owned(),
        stages: plan
            .stages
            .iter()
            .map(|row| PlanStageView {
                id: row.id.clone(),
                title: row.title.clone(),
                hours: span(row.hours),
            })
            .collect(),
        children: plan
            .children
            .iter()
            .map(|row| PlanPartView {
                title: row.title.clone(),
                goal: row.goal.clone(),
                hours: span(row.hours),
            })
            .collect(),
    }
}

fn span(hours: Hours) -> Span {
    Span {
        min: hours.min,
        max: hours.max,
    }
}

fn hours(span: &Span) -> Hours {
    Hours {
        min: span.min,
        max: span.max,
    }
}

fn refused(error: GenerateError) -> IpcError {
    IpcError::new(error.code(), error.to_string())
}
