use osu_db::Replay;
use serenity::model::channel::Attachment;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub enum AttachmentParseResult {
    NoAttachmentOrReplay,
    BeingProcessed,
    FailedDownload(serenity::prelude::SerenityError),
    FailedParsing(osu_db::Error),
}

pub async fn process_replay(mut receiver: UnboundedReceiver<Replay>) {
    loop {
        let parsed_replay = receiver.recv().await.unwrap();
        // ...
    }
}

pub async fn parse_attachment_replay(
    attachments: &[Attachment],
    sender: &UnboundedSender<Replay>,
) -> AttachmentParseResult {
    let attachment = match attachments.last() {
        Some(a) => a,
        None => return AttachmentParseResult::NoAttachmentOrReplay,
    };

    let file_type = match attachment.filename.split('.').last() {
        Some(a) => a,
        None => return AttachmentParseResult::NoAttachmentOrReplay,
    };

    if file_type != "osr" {
        return AttachmentParseResult::NoAttachmentOrReplay;
    }

    match attachment.download().await {
        // parse the data as replay
        Ok(bytes) => match osu_db::Replay::from_bytes(&bytes) {
            Ok(replay) => {
                sender.send(replay);
                return AttachmentParseResult::BeingProcessed;
            }
            Err(why) => AttachmentParseResult::FailedParsing(why),
        },
        Err(why) => AttachmentParseResult::FailedDownload(why),
    }
}
