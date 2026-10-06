#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "generate gate: a panic here is the report"
)]

use std::cell::{Cell, RefCell};
use std::time::Duration;

use tolearn_generate::ledger::Kind;
use tolearn_generate::{GenerateError, HOSTS, Model, Step, online};
use tolearn_offline::reach::Reach;
use tolearn_provider::{CheckError, Said, Tokens};

#[derive(Debug, Default)]
struct Net {
    down: Option<&'static str>,
    asked: RefCell<Vec<String>>,
}

impl Reach for Net {
    fn reach(&self, url: &str) -> Result<(), String> {
        self.asked.borrow_mut().push(url.to_owned());
        match self.down {
            Some(host) if url.contains(host) => Err("connection refused".to_owned()),
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Default)]
struct Counting {
    calls: Cell<usize>,
    fails: Option<CheckError>,
    pause: Duration,
}

impl Model for Counting {
    fn ask(&self, _prompt: &str) -> Result<Said, CheckError> {
        self.calls.set(self.calls.get() + 1);
        std::thread::sleep(self.pause);
        match &self.fails {
            Some(error) => Err(error.clone()),
            None => Ok(Said {
                text: "карта".to_owned(),
                thinking: false,
                tokens: Tokens::default(),
                model: None,
            }),
        }
    }
}

#[test]
fn sources_are_checked_at_open_library_and_commons() {
    assert_eq!(HOSTS, ["openlibrary.org", "commons.wikimedia.org"]);
}

#[test]
fn without_network_the_model_is_never_called() {
    for host in HOSTS {
        let net = Net {
            down: Some(host),
            ..Net::default()
        };
        let model = Counting::default();

        let error = online(&net, &model)
            .and_then(|gate| gate.ask(Step::Plan, "карта"))
            .unwrap_err();

        assert_eq!(error.code(), "generate.offline");
        assert!(
            matches!(&error, GenerateError::Offline { host: named, .. } if named == host),
            "{error:?}"
        );
        assert!(error.to_string().contains(host), "{error}");
        assert!(error.to_string().contains("connection refused"), "{error}");
        assert_eq!(model.calls.get(), 0, "{host}");
    }
}

#[test]
fn every_host_is_checked_before_the_model() {
    let net = Net::default();
    let model = Counting::default();

    let said = online(&net, &model)
        .unwrap()
        .ask(Step::Plan, "карта")
        .unwrap();

    assert_eq!(said.text, "карта");
    assert_eq!(model.calls.get(), 1);
    let asked = net.asked.borrow();
    for host in HOSTS {
        assert!(asked.iter().any(|url| url.contains(host)), "{asked:?}");
    }
}

#[test]
fn a_provider_failure_keeps_its_own_code() {
    let net = Net::default();
    let model = Counting {
        fails: Some(CheckError::Unreachable("timeout".to_owned())),
        ..Counting::default()
    };

    let error = online(&net, &model)
        .unwrap()
        .ask(Step::Plan, "карта")
        .unwrap_err();

    assert_eq!(error.code(), "provider.unreachable");
    assert_ne!(error.code(), "generate.offline");
    assert_eq!(
        error,
        GenerateError::Provider(CheckError::Unreachable("timeout".to_owned()))
    );
}

#[test]
fn every_call_is_timed_into_the_tally_with_its_step_and_round_even_a_cancelled_one() {
    let net = Net::default();
    let slow = Counting {
        pause: Duration::from_millis(30),
        ..Counting::default()
    };
    let gate = online(&net, &slow).unwrap();
    gate.ask(Step::Plan, "карта").unwrap();
    gate.ask(Step::Repair(2), "карта").unwrap();
    gate.again(Step::Sources, 1, "карта").unwrap();

    let records = gate.tally().records();
    let steps: Vec<(&str, Option<usize>)> = records
        .iter()
        .map(|record| (record.step.as_str(), record.round))
        .collect();
    assert_eq!(
        steps,
        [("plan", None), ("repair", Some(2)), ("sources", Some(1))]
    );
    assert!(
        records.iter().all(|record| record.ok
            && record.kind == Kind::Model
            && record.ms >= 30
            && record.input.is_none()
            && record.output.is_none()),
        "{records:?}"
    );

    let failing = Counting {
        fails: Some(CheckError::Unreachable("timeout".to_owned())),
        ..Counting::default()
    };
    let gate = online(&net, &failing).unwrap();
    gate.ask(Step::Text, "этап").unwrap_err();
    let refused = gate.tally().records();
    assert_eq!(refused.len(), 1);
    assert_eq!(refused[0].step, "text");
    assert!(!refused[0].ok);

    let cancelled = Counting {
        fails: Some(CheckError::Cancelled),
        ..Counting::default()
    };
    let gate = online(&net, &cancelled).unwrap();
    assert_eq!(
        gate.ask(Step::Text, "этап").unwrap_err(),
        GenerateError::Cancelled
    );
    let stopped = gate.tally().records();
    assert_eq!(stopped.len(), 1);
    assert_eq!(stopped[0].step, "text");
    assert!(!stopped[0].ok);
}
