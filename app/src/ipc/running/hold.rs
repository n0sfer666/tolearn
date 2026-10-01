use super::super::error::IpcError;
use super::super::generating::GenerationWork;

#[derive(Debug, Clone)]
pub enum Hold {
    Generation(Box<GenerationWork>),
    Exam(String),
    Clarify(String),
}

impl Hold {
    pub fn generation(work: GenerationWork) -> Self {
        Self::Generation(Box::new(work))
    }

    pub(super) fn program(&self) -> Option<&str> {
        match self {
            Self::Generation(work) => (!work.program.is_empty()).then_some(work.program.as_str()),
            Self::Exam(program) | Self::Clarify(program) => Some(program),
        }
    }

    pub(super) fn work(&self) -> Option<&GenerationWork> {
        match self {
            Self::Generation(work) => Some(work),
            Self::Exam(_) | Self::Clarify(_) => None,
        }
    }

    pub(super) fn work_mut(&mut self) -> Option<&mut GenerationWork> {
        match self {
            Self::Generation(work) => Some(work),
            Self::Exam(_) | Self::Clarify(_) => None,
        }
    }

    pub(super) fn busy(&self) -> IpcError {
        let (by, message) = match self {
            Self::Generation(_) => (
                "generation",
                "уже идёт генерация: дождитесь её или отмените",
            ),
            Self::Exam(_) => ("exam", "идёт проверка ответа: дождитесь её"),
            Self::Clarify(_) => ("clarify", "идёт уточнение: дождитесь его"),
        };
        IpcError::new("generate.busy", message.to_owned()).holding(by)
    }
}
