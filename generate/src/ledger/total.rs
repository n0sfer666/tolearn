use super::kind::Kind;
use super::record::Record;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Total {
    pub ms: u64,
    pub input: u64,
    pub output: u64,
    pub calls: usize,
    pub checks: usize,
    pub unknown: usize,
}

impl Total {
    pub fn of<'a>(records: impl IntoIterator<Item = &'a Record>) -> Self {
        records
            .into_iter()
            .fold(Self::default(), |mut total, record| {
                total.ms += record.ms;
                if record.kind == Kind::Model {
                    total.calls += 1;
                    total.input += u64::from(record.input.unwrap_or_default());
                    total.output += u64::from(record.output.unwrap_or_default());
                    if record.input.is_none() || record.output.is_none() {
                        total.unknown += 1;
                    }
                } else {
                    total.checks += 1;
                }
                total
            })
    }

    pub fn stage<'a>(
        records: impl IntoIterator<Item = &'a Record>,
        program: &str,
        stage: &str,
    ) -> Self {
        Self::of(records.into_iter().filter(|record| {
            record.program.as_deref() == Some(program) && record.stage.as_deref() == Some(stage)
        }))
    }

    pub fn complete(&self) -> bool {
        self.unknown == 0
    }
}
