use std::fmt;
use std::sync::Arc;

use tolearn_generate::Progress;
use tolearn_generate::diagram::Painter;
use tolearn_offline::page::{AsFetched, Prerenderer, Source, Web};

use super::announced::Announced;
use super::error::IpcError;
use super::started::GenerationStep;
use super::unpainted::Unpainted;

const FETCH_SECS: u64 = 30;

pub type Fetcher = Arc<dyn Source + Send + Sync>;
pub type Renderer = Arc<dyn Prerenderer + Send + Sync>;
pub type Brush = Arc<dyn Painter + Send + Sync>;
pub type Herald = Arc<dyn Fn(GenerationStep) + Send + Sync>;

#[derive(Clone, Default)]
pub struct Tools {
    pub source: Option<Fetcher>,
    pub renderer: Option<Renderer>,
    pub painter: Option<Brush>,
    pub herald: Option<Herald>,
}

impl Tools {
    pub fn fetcher(&self) -> Result<Fetcher, IpcError> {
        if let Some(source) = &self.source {
            return Ok(Arc::clone(source));
        }
        let web =
            Web::new(FETCH_SECS).map_err(|reason| IpcError::new("generate.offline", reason))?;
        Ok(Arc::new(web))
    }

    pub fn rendering(&self) -> Renderer {
        self.renderer.clone().unwrap_or_else(|| Arc::new(AsFetched))
    }

    pub fn painting(&self) -> Brush {
        self.painter.clone().unwrap_or_else(|| Arc::new(Unpainted))
    }

    pub fn progress(&self) -> Box<dyn Progress> {
        match &self.herald {
            Some(herald) => Box::new(Announced::new(Arc::clone(herald))),
            None => Box::new(()),
        }
    }
}

impl fmt::Debug for Tools {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.debug_struct("Tools")
            .field("source", &self.source.is_some())
            .field("renderer", &self.renderer.is_some())
            .field("painter", &self.painter.is_some())
            .field("herald", &self.herald.is_some())
            .finish()
    }
}
