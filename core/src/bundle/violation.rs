use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    EmptyStages,
    EmptyTopics,
    DuplicateTopicId {
        id: String,
    },
    StageOutOfRange {
        topic: String,
        stage: u32,
    },
    UnknownCheckpoint {
        stage: u32,
        checkpoint: String,
    },
    CheckpointOutsideStage {
        stage: u32,
        checkpoint: String,
        found: u32,
    },
    MissingTopicFile {
        topic: String,
        stage: u32,
        file: String,
    },
    UnsafeTopicFile {
        topic: String,
        file: String,
    },
    DuplicateTopicFile {
        file: String,
    },
    SchemaMajorMismatch {
        topic: String,
        roadmap: u32,
        found: u32,
    },
    HoursReversed {
        topic: String,
        min: u32,
        max: u32,
    },
    UnknownDependency {
        topic: String,
        depends_on: String,
    },
    ForwardDependency {
        topic: String,
        depends_on: String,
    },
    DependencyCycle {
        chain: Vec<String>,
    },
    ProgressForAnotherProgram {
        program: String,
        found: String,
    },
    UntrackedTopic {
        topic: String,
    },
    StrayProgressTopic {
        topic: String,
    },
}

impl Violation {
    pub fn code(&self) -> &'static str {
        match self {
            Self::EmptyStages => "bundle.empty-stages",
            Self::EmptyTopics => "bundle.empty-topics",
            Self::DuplicateTopicId { .. } => "bundle.duplicate-id",
            Self::StageOutOfRange { .. } => "bundle.stage-out-of-range",
            Self::UnknownCheckpoint { .. } => "bundle.unknown-checkpoint",
            Self::CheckpointOutsideStage { .. } => "bundle.checkpoint-outside-stage",
            Self::MissingTopicFile { .. } => "bundle.missing-topic-file",
            Self::UnsafeTopicFile { .. } => "bundle.unsafe-file",
            Self::DuplicateTopicFile { .. } => "bundle.duplicate-file",
            Self::SchemaMajorMismatch { .. } => "bundle.schema-major-mismatch",
            Self::HoursReversed { .. } => "bundle.hours-reversed",
            Self::UnknownDependency { .. } => "bundle.unknown-dependency",
            Self::ForwardDependency { .. } => "bundle.forward-dependency",
            Self::DependencyCycle { .. } => "bundle.cycle",
            Self::ProgressForAnotherProgram { .. } => "bundle.progress-elsewhere",
            Self::UntrackedTopic { .. } => "bundle.untracked-topic",
            Self::StrayProgressTopic { .. } => "bundle.stray-progress-topic",
        }
    }

    pub fn topics(&self) -> Vec<&str> {
        match self {
            Self::EmptyStages
            | Self::EmptyTopics
            | Self::DuplicateTopicFile { .. }
            | Self::ProgressForAnotherProgram { .. } => Vec::new(),
            Self::DuplicateTopicId { id } => vec![id],
            Self::UnknownCheckpoint { checkpoint, .. }
            | Self::CheckpointOutsideStage { checkpoint, .. } => vec![checkpoint],
            Self::StageOutOfRange { topic, .. }
            | Self::MissingTopicFile { topic, .. }
            | Self::UnsafeTopicFile { topic, .. }
            | Self::SchemaMajorMismatch { topic, .. }
            | Self::HoursReversed { topic, .. }
            | Self::UnknownDependency { topic, .. }
            | Self::ForwardDependency { topic, .. }
            | Self::UntrackedTopic { topic }
            | Self::StrayProgressTopic { topic } => vec![topic],
            Self::DependencyCycle { chain } => chain.iter().map(String::as_str).collect(),
        }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: ", self.code())?;
        match self {
            Self::EmptyStages => write!(formatter, "the program has no stages"),
            Self::EmptyTopics => write!(formatter, "the program has no topics"),
            Self::DuplicateTopicId { id } => {
                write!(formatter, "`{id}` is the id of more than one topic")
            }
            Self::StageOutOfRange { topic, stage } => write!(
                formatter,
                "`{topic}` belongs to stage {stage}, and the program has no such stage"
            ),
            Self::UnknownCheckpoint { stage, checkpoint } => write!(
                formatter,
                "stage {stage} names `{checkpoint}` as its checkpoint, and the program has no such topic"
            ),
            Self::CheckpointOutsideStage {
                stage,
                checkpoint,
                found,
            } => write!(
                formatter,
                "stage {stage} is checked by `{checkpoint}`, which belongs to stage {found}"
            ),
            Self::MissingTopicFile { topic, stage, file } => write!(
                formatter,
                "stage {stage} is generated, and `{topic}` has no document at {file}"
            ),
            Self::UnsafeTopicFile { topic, file } => write!(
                formatter,
                "`{topic}` is kept at {file}, which leads outside the program directory"
            ),
            Self::DuplicateTopicFile { file } => {
                write!(formatter, "{file} is the document of more than one topic")
            }
            Self::SchemaMajorMismatch {
                topic,
                roadmap,
                found,
            } => write!(
                formatter,
                "`{topic}` is written against schema major {found}, and the program against {roadmap}"
            ),
            Self::HoursReversed { topic, min, max } => write!(
                formatter,
                "`{topic}` is estimated at {min} hours at the least and {max} at the most"
            ),
            Self::UnknownDependency { topic, depends_on } => write!(
                formatter,
                "`{topic}` depends on `{depends_on}`, and the program has no such topic"
            ),
            Self::ForwardDependency { topic, depends_on } => write!(
                formatter,
                "`{topic}` depends on `{depends_on}`, which the plan puts no earlier than `{topic}` itself"
            ),
            Self::DependencyCycle { chain } => {
                write!(formatter, "the dependencies close a cycle: ")?;
                for id in chain {
                    write!(formatter, "{id} -> ")?;
                }
                write!(formatter, "{}", chain.first().map_or("", String::as_str))
            }
            Self::ProgressForAnotherProgram { program, found } => write!(
                formatter,
                "the progress belongs to `{found}`, and the program is `{program}`"
            ),
            Self::UntrackedTopic { topic } => {
                write!(formatter, "`{topic}` has no entry in the progress")
            }
            Self::StrayProgressTopic { topic } => write!(
                formatter,
                "the progress tracks `{topic}`, and the program has no such topic"
            ),
        }
    }
}
