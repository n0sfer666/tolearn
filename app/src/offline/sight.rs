use tolearn_offline::fresh::WINDOW;
use tolearn_offline::queue::Strategy;
use tolearn_offline::store::Held;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sight {
    Fetch,
    Ask,
    Fresh,
    Unchecked,
}

pub fn sight(how: Strategy, held: Option<&Held>, at: i64) -> Sight {
    let Some(held) = held else {
        return Sight::Fetch;
    };
    if !how.checkable() {
        return Sight::Unchecked;
    }
    match held.checked_at.is_some_and(|checked| at - checked < WINDOW) {
        true => Sight::Fresh,
        false => Sight::Ask,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn held(checked_at: Option<i64>) -> Held {
        Held {
            hash: "hash".to_owned(),
            path: PathBuf::new(),
            kind: "archive".to_owned(),
            size: 1024,
            fetched_at: 1_700_000_000,
            etag: None,
            last_modified: None,
            body_hash: None,
            checked_at,
        }
    }

    #[test]
    fn нескачанное_качается_каким_бы_оно_ни_было() {
        assert_eq!(sight(Strategy::Archive, None, 100), Sight::Fetch);
        assert_eq!(sight(Strategy::Clone, None, 100), Sight::Fetch);
        assert_eq!(sight(Strategy::Video, None, 100), Sight::Fetch);
    }

    #[test]
    fn скачанное_непроверяемое_не_трогается() {
        let kept = held(None);

        assert_eq!(sight(Strategy::Clone, Some(&kept), 100), Sight::Unchecked);
        assert_eq!(sight(Strategy::Video, Some(&kept), 100), Sight::Unchecked);
    }

    #[test]
    fn свежее_в_окне_не_идёт_в_сеть() {
        let kept = held(Some(1_000));

        assert_eq!(
            sight(Strategy::Archive, Some(&kept), 1_000 + WINDOW - 1),
            Sight::Fresh
        );
    }

    #[test]
    fn устаревшее_идёт_условным_запросом() {
        let kept = held(Some(1_000));

        assert_eq!(
            sight(Strategy::Archive, Some(&kept), 1_000 + WINDOW),
            Sight::Ask
        );
        assert_eq!(sight(Strategy::Mirror, Some(&held(None)), 100), Sight::Ask);
    }
}
