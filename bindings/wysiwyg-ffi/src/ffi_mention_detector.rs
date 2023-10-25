use std::sync::Arc;

#[derive(Default, uniffi::Object)]
pub struct MentionDetector {}

impl MentionDetector {
    pub fn new() -> Self {
        Self {}
    }
}

#[uniffi::export]
impl MentionDetector {
    pub fn is_mention(self: &Arc<Self>, url: String) -> bool {
        matrix_mentions::is_mention(&url)
    }
}
