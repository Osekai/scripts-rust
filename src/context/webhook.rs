use eyre::Result;

use crate::model::{Finish, Progress};

use super::Context;

impl Context {
    pub async fn handle_progress(&self, progress: &Progress) -> Result<()> {
        if let Err(err) = self.client.notify_webhook_progress(progress).await {
            error!(?err, "Failed to notify webhook of progress");
        }

        self.mysql.store_progress(progress).await
    }

    pub async fn handle_finish(&self, finish: Finish) -> Result<()> {
        if let Err(err) = self.client.notify_webhook_finish(&finish).await {
            error!(?err, "Failed to notify webhook of finish");
        }

        self.mysql.store_finish(&finish).await
    }
}
