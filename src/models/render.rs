use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct RenderedPage {
    pub slug: String,
    pub html: String,
}
