use tolearn_core::program::Sources;
use tolearn_core::stage::Stage;

use super::assemble::Assets;

pub(crate) struct Built {
    pub(crate) stage: Stage,
    pub(crate) assets: Assets,
    pub(crate) cited: Sources,
}
