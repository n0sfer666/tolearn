#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "generate gate: a panic here is the report"
)]

use std::cell::{Cell, RefCell};

use tolearn_generate::{GenerateError, HOSTS, Model, online};
use tolearn_offline::reach::Reach;
use tolearn_provider::{CheckError, Said};

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
}

impl Model for Counting {
    fn ask(&self, _prompt: &str) -> Result<Said, CheckError> {
        self.calls.set(self.calls.get() + 1);
        match &self.fails {
            Some(error) => Err(error.clone()),
            None => Ok(Said {
                text: "карта".to_owned(),
                thinking: false,
                tokens: None,
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
            .and_then(|gate| gate.ask("карта"))
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

    let said = online(&net, &model).unwrap().ask("карта").unwrap();

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

    let error = online(&net, &model).unwrap().ask("карта").unwrap_err();

    assert_eq!(error.code(), "provider.unreachable");
    assert_ne!(error.code(), "generate.offline");
    assert_eq!(
        error,
        GenerateError::Provider(CheckError::Unreachable("timeout".to_owned()))
    );
}
