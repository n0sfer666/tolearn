use std::fmt;
use std::path::Path;

use tolearn_offline::page::{Prerenderer, Source};
use tolearn_offline::store::Store;
use tolearn_provider::Stop;

use crate::diagram::Painter;
use crate::gate::Online;
use crate::progress::Progress;

pub struct Kit<'a> {
    pub online: &'a Online<'a>,
    pub source: &'a dyn Source,
    pub renderer: &'a dyn Prerenderer,
    pub store: &'a mut Store,
    pub painter: &'a dyn Painter,
    pub progress: &'a dyn Progress,
    pub stop: &'a Stop,
    pub data: &'a Path,
    pub at: i64,
}

impl fmt::Debug for Kit<'_> {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.debug_struct("Kit")
            .field("online", self.online)
            .field("store", &self.store)
            .field("stop", self.stop)
            .field("data", &self.data)
            .field("at", &self.at)
            .finish_non_exhaustive()
    }
}
