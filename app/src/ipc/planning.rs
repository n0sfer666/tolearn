use tolearn_core::Hours;
use tolearn_core::program::{StageRow, Volatility};
use tolearn_core::settings::Locale;
use tolearn_generate::ledger::{Record, Tally};
use tolearn_generate::plan::{Part, Plan, Request};
use tolearn_generate::{GenerateError, Online, Step, online, stepped};
use tolearn_provider::{Provider, Stop};

use super::context::Context;
use super::error::IpcError;
use super::planned::{PlanOut, PlanPartView, PlanStageView, PlanView};
use super::provider::{denied, failed};
use super::types::Span;
use super::voiced::Voiced;
use crate::journal::Journal;

pub fn drawn(
    context: &Context,
    kind: &'static str,
    step: Step,
    request: &Request,
    keep: fn(&Tally, Vec<Record>),
    draw: impl FnOnce(&Online<'_>, &Request) -> Result<Plan, GenerateError>,
) -> Result<PlanOut, IpcError> {
    let model = voiced(context, kind, Stop::default())?;
    let reach = context.reach()?;
    let online = online(reach.as_ref(), &model).map_err(refused)?;
    let ticket = context.book().ticket();
    let progress = context.tools().progress();
    let drawn = stepped(progress.as_ref(), step, || draw(&online, request));
    let spent = online.tally().take();
    let plan = match drawn {
        Ok(plan) => {
            context.book().settle(ticket, spent, keep);
            plan
        }
        Err(error) => {
            context.ledger().extend(spent);
            return Err(refused(error));
        }
    };
    Ok(PlanOut {
        hours: span(plan.hours()),
        plan: view(&plan),
    })
}

pub fn asked(request: &str, level: &str, locale: &str) -> Result<Request, IpcError> {
    let tongue = Locale::parse(locale).ok_or_else(|| {
        IpcError::new(
            "plan.unknown-value",
            format!("`{locale}` — не язык программы"),
        )
    })?;
    Ok(Request {
        request: told("запрос", request)?,
        level: level.trim().to_owned(),
        locale: tongue.label().to_owned(),
    })
}

pub fn voiced(context: &Context, kind: &'static str, stop: Stop) -> Result<Voiced, IpcError> {
    let provider = Provider::read(&context.provider()).map_err(failed)?;
    let key = context.vault().key().map_err(denied)?;
    let journal = Journal::new(context.llm_log(), provider.journal);
    Ok(Voiced::new(provider, key, journal, kind, stop))
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

pub fn refused(error: GenerateError) -> IpcError {
    IpcError::new(error.code(), error.to_string())
}
