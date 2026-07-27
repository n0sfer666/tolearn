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
    SchemaMajorMismatch {
        topic: String,
        roadmap: u32,
        found: u32,
    },
    UnknownDependency {
        topic: String,
        depends_on: String,
    },
    DependencyCycle {
        chain: Vec<String>,
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
            Self::SchemaMajorMismatch { .. } => "bundle.schema-major-mismatch",
            Self::UnknownDependency { .. } => "bundle.unknown-dependency",
            Self::DependencyCycle { .. } => "bundle.cycle",
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
            Self::SchemaMajorMismatch {
                topic,
                roadmap,
                found,
            } => write!(
                formatter,
                "`{topic}` is written against schema major {found}, and the program against {roadmap}"
            ),
            Self::UnknownDependency { topic, depends_on } => write!(
                formatter,
                "`{topic}` depends on `{depends_on}`, and the program has no such topic"
            ),
            Self::DependencyCycle { chain } => {
                write!(formatter, "the dependencies close a cycle: ")?;
                for id in chain {
                    write!(formatter, "{id} -> ")?;
                }
                write!(formatter, "{}", chain.first().map_or("", String::as_str))
            }
        }
    }
}
