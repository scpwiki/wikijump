//! Corpus render finalization and inventory workflows.

mod finalizer;
mod inventory;
mod inventory_query;
#[cfg(test)]
mod inventory_tests;

pub use self::finalizer::{
    CorpusRenderFinalizerService, RenderFinalizerPass, RenderFinalizerSettings,
    RenderFinalizerSummary,
};
pub use self::inventory::{
    CorpusRenderInventoryService, RenderInventoryPass, RenderInventorySettings,
    RenderInventorySummary,
};
