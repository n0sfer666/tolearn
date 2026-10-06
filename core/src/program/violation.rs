use std::fmt;

use super::MAX_DEPTH;
use crate::block::Kind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    TooDeep {
        program: String,
    },
    NodeWithStages {
        program: String,
    },
    DuplicateUuid {
        uuid: String,
    },
    ChildUuidMismatch {
        row: String,
        found: String,
    },
    StageIdMismatch {
        row: String,
        found: String,
    },
    HoursReversed {
        row: String,
        min: u32,
        max: u32,
    },
    BlockId {
        stage: String,
        block: String,
        expected: String,
    },
    BlockWithoutAsset {
        stage: String,
        block: String,
        kind: Kind,
    },
    ForeignBlockField {
        stage: String,
        block: String,
        kind: Kind,
        field: &'static str,
    },
    UnlicensedImage {
        stage: String,
        block: String,
    },
    MissingAsset {
        stage: String,
        block: String,
        asset: String,
    },
}

impl Violation {
    pub fn code(&self) -> &'static str {
        match self {
            Self::TooDeep { .. } => "program.too-deep",
            Self::NodeWithStages { .. } => "program.node-with-stages",
            Self::DuplicateUuid { .. } => "program.duplicate-uuid",
            Self::ChildUuidMismatch { .. } => "program.child-uuid-mismatch",
            Self::StageIdMismatch { .. } => "program.stage-id-mismatch",
            Self::HoursReversed { .. } => "program.hours-reversed",
            Self::BlockId { .. } => "program.block-id",
            Self::BlockWithoutAsset { .. } => "program.block-without-asset",
            Self::ForeignBlockField { .. } => "program.foreign-block-field",
            Self::UnlicensedImage { .. } => "program.unlicensed-image",
            Self::MissingAsset { .. } => "program.missing-asset",
        }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: ", self.code())?;
        match self {
            Self::TooDeep { program } => write!(
                formatter,
                "`{program}` stands {MAX_DEPTH} levels deep and still holds subprograms"
            ),
            Self::NodeWithStages { program } => {
                write!(formatter, "`{program}` holds both stages and subprograms")
            }
            Self::DuplicateUuid { uuid } => {
                write!(
                    formatter,
                    "`{uuid}` names more than one program of the tree"
                )
            }
            Self::ChildUuidMismatch { row, found } => {
                write!(formatter, "children/{row} holds the program `{found}`")
            }
            Self::StageIdMismatch { row, found } => {
                write!(formatter, "stages/{row}.yaml holds the stage `{found}`")
            }
            Self::HoursReversed { row, min, max } => write!(
                formatter,
                "`{row}` is estimated at {min} hours at the least and {max} at the most"
            ),
            Self::BlockId {
                stage,
                block,
                expected,
            } => write!(
                formatter,
                "block `{block}` of `{stage}` is `{expected}` by its text"
            ),
            Self::BlockWithoutAsset { stage, block, kind } => write!(
                formatter,
                "the {} `{block}` of `{stage}` names no asset",
                kind.label()
            ),
            Self::ForeignBlockField {
                stage,
                block,
                kind,
                field,
            } => write!(
                formatter,
                "the {} `{block}` of `{stage}` carries `{field}`, which that kind has no use for",
                kind.label()
            ),
            Self::UnlicensedImage { stage, block } => write!(
                formatter,
                "the image `{block}` of `{stage}` lacks a license or an attribution"
            ),
            Self::MissingAsset {
                stage,
                block,
                asset,
            } => write!(
                formatter,
                "block `{block}` of `{stage}` shows {asset}, which the program does not hold"
            ),
        }
    }
}
