use tolearn_offline::book::Wanted;
use tolearn_provider::Stop;

use crate::error::GenerateError;
use crate::gate::Online;
use crate::halt;
use crate::sources::{Sources, Verdict};

use super::gathered::{Chaptered, Dropped, Gathered, Visited};
use super::place::Place;
use super::prompt;
use super::proposal::{self, Proposal};
use super::{MAX_BOOKS, MAX_IMAGES, MAX_PAGES};

const UNREAD: &str = "ответ модели";

pub fn gather(
    online: &Online<'_>,
    sources: &mut Sources<'_>,
    place: &Place<'_>,
    stop: &Stop,
) -> Result<Gathered, GenerateError> {
    let task = prompt::sources(place);
    let said = online.ask(&task)?;
    let mut gathered = Gathered::default();
    let refused = checked(sources, proposal::read(&said.text), &mut gathered, stop)?;
    if refused.is_empty() {
        return Ok(gathered);
    }
    let again = online.ask(&prompt::replace(&task, &said.text, &refused))?;
    gathered.dropped = refused;
    let refused = checked(sources, proposal::read(&again.text), &mut gathered, stop)?;
    gathered.dropped.extend(refused);
    Ok(gathered)
}

fn checked(
    sources: &mut Sources<'_>,
    proposal: Result<Proposal, String>,
    gathered: &mut Gathered,
    stop: &Stop,
) -> Result<Vec<Dropped>, GenerateError> {
    let proposal = match proposal {
        Ok(proposal) => proposal,
        Err(reason) => {
            return Ok(vec![Dropped {
                what: UNREAD.to_owned(),
                reason,
            }]);
        }
    };
    let mut dropped = Vec::new();
    for wish in proposal.books {
        if gathered.books.len() >= MAX_BOOKS {
            break;
        }
        let wanted = Wanted {
            isbn: wish.isbn.filter(|isbn| !isbn.trim().is_empty()),
            title: wish.title.clone(),
            author: wish.author,
        };
        halt::checked(stop)?;
        let outcome = sources.book(&wanted)?;
        match outcome.verdict {
            Verdict::Passed(book) => {
                if !gathered.books.iter().any(|known| known.book == book) {
                    gathered.books.push(Chaptered {
                        book,
                        chapter: wish.chapter,
                        checked_at: outcome.checked_at,
                    });
                }
            }
            Verdict::Refused(reason) => dropped.push(Dropped {
                what: format!("книга «{}»", wish.title),
                reason,
            }),
        }
    }
    for wish in proposal.pages {
        if gathered.pages.len() >= MAX_PAGES || gathered.visited(&wish.url) {
            continue;
        }
        halt::checked(stop)?;
        let outcome = sources.page(&wish.url)?;
        match outcome.verdict {
            Verdict::Passed(page) => gathered.pages.push(Visited {
                page,
                checked_at: outcome.checked_at,
            }),
            Verdict::Refused(reason) => dropped.push(Dropped {
                what: format!("страница {}", wish.url),
                reason,
            }),
        }
    }
    for wish in proposal.images {
        if gathered.images.len() >= MAX_IMAGES {
            break;
        }
        halt::checked(stop)?;
        match sources.image(&wish.query, &wish.caption).verdict {
            Verdict::Passed(image) => {
                if !gathered.images.iter().any(|known| known.file == image.file) {
                    gathered.images.push(image);
                }
            }
            Verdict::Refused(reason) => dropped.push(Dropped {
                what: format!("картинка «{}»", wish.query),
                reason,
            }),
        }
    }
    Ok(dropped)
}
