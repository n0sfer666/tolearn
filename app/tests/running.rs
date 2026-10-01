#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use support::starter::{Case, LEVEL, REQUEST, flat, told};
use tolearn_app::ipc::{Finale, GenerationStateOut, GenerationWork, Hold, IpcError, Running};

const OTHER: &str = "9f1c0d33-4b7a-4e2f-8c15-6a3d9e0b7f42";

fn heard(running: &Running) -> Arc<Mutex<Vec<GenerationStateOut>>> {
    let heard = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&heard);
    running.herald(Arc::new(move |told| seen.lock().unwrap().push(told)));
    heard
}

#[test]
fn после_события_конца_состояние_уже_отдаёт_итог() {
    let running = Running::default();
    let probe = running.clone();
    let heard = Arc::new(Mutex::new(
        Vec::<(GenerationStateOut, GenerationStateOut)>::new(),
    ));
    let seen = Arc::clone(&heard);
    running.herald(Arc::new(move |told| {
        seen.lock().unwrap().push((told, probe.view()));
    }));
    let mut case = Case::new(true, told(flat()));
    case.context = case.context.clone().with_running(running);

    case.plan();

    let heard = heard.lock().unwrap().clone();
    assert_eq!(heard.len(), 2);
    let (begun, _) = &heard[0];
    assert!(begun.work.as_ref().is_some_and(|work| work.kind == "plan"));
    assert!(begun.outcome.is_none());
    let (ended, then) = &heard[1];
    assert!(ended.work.is_none());
    assert!(
        ended
            .outcome
            .as_ref()
            .is_some_and(|done| done.plan.is_some())
    );
    assert_eq!(ended, then);
}

#[test]
fn паника_потока_кончает_работу_отказом_и_событием() {
    let running = Running::default();
    let heard = heard(&running);
    let worker = running.clone();

    let fell = std::thread::spawn(move || {
        let work = GenerationWork::asked("plan", REQUEST, LEVEL, "ru");
        let _claim = worker.claim(Hold::generation(work)).unwrap();
        panic!("поток генерации упал");
    })
    .join();

    assert!(fell.is_err());
    let now = running.view();
    assert!(now.work.is_none());
    let refusal = now.outcome.as_ref().and_then(|done| done.refusal.as_ref());
    assert_eq!(
        refusal.map(|refusal| refusal.code.as_str()),
        Some("ipc.crashed")
    );
    let heard = heard.lock().unwrap().clone();
    assert_eq!(heard.len(), 2);
    assert_eq!(heard[1], now);
}

#[test]
fn следующая_генерация_снимает_прежний_итог_а_зачёт_его_не_трогает() {
    let running = Running::default();
    let refused = IpcError::new("generate.offline", "нет сети".to_owned());
    let plan = GenerationWork::asked("plan", REQUEST, LEVEL, "ru");
    running
        .claim(Hold::generation(plan))
        .unwrap()
        .end(Some(Finale::Refused(refused)));
    let waiting = running.view();

    drop(running.claim(Hold::Exam(OTHER.to_owned())).unwrap());
    let after_exam = running.view();
    let fork = GenerationWork::placed("fork", OTHER, "", "tracker");
    let claim = running.claim(Hold::generation(fork)).unwrap();
    let during = running.view();
    drop(claim);

    assert!(waiting.outcome.is_some());
    assert_eq!(after_exam, waiting);
    assert!(during.outcome.is_none());
    assert!(during.work.is_some_and(|work| work.kind == "fork"));
    assert_eq!(
        running.view(),
        GenerationStateOut {
            work: None,
            outcome: None
        }
    );
}

#[test]
fn событие_снятого_удалением_итога_не_перекрывает_начатую_следом_генерацию() {
    let running = Running::default();
    let heard = Arc::new(Mutex::new(Vec::<GenerationStateOut>::new()));
    let seen = Arc::clone(&heard);
    let (entered, waiting) = mpsc::channel::<()>();
    let entered = Mutex::new(entered);
    let calls = AtomicUsize::new(0);
    running.herald(Arc::new(move |told| {
        if calls.fetch_add(1, Ordering::SeqCst) == 2 {
            entered.lock().unwrap().send(()).unwrap();
            std::thread::sleep(Duration::from_millis(200));
        }
        seen.lock().unwrap().push(told);
    }));
    let refused = IpcError::new("generate.offline", "нет сети".to_owned());
    let work = GenerationWork::placed("fork", OTHER, "", "tracker");
    running
        .claim(Hold::generation(work))
        .unwrap()
        .end(Some(Finale::Refused(refused)));
    let erasing = running.erasing(OTHER).unwrap();

    let gone = std::thread::spawn(move || erasing.gone());
    waiting.recv_timeout(Duration::from_secs(5)).unwrap();
    let next = GenerationWork::asked("plan", REQUEST, LEVEL, "ru");
    let _claim = running.claim(Hold::generation(next)).unwrap();
    gone.join().unwrap();

    let heard = heard.lock().unwrap().clone();
    assert_eq!(heard.len(), 4);
    assert_eq!(heard.last(), Some(&running.view()));
}
