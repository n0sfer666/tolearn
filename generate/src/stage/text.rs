use crate::error::GenerateError;
use crate::gate::Online;

use super::answer;
use super::draft::Drafted;
use super::gathered::Gathered;
use super::place::Place;
use super::prompt;

pub fn text(
    online: &Online<'_>,
    place: &Place<'_>,
    gathered: &Gathered,
) -> Result<Drafted, GenerateError> {
    let said = online.ask(&prompt::text(place, gathered))?;
    let draft = answer::read(&said.text, place, gathered);
    Ok(Drafted {
        said: said.text,
        draft,
    })
}
