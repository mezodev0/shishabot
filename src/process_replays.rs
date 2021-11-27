use osu_db::Replay;
use rosu_v2::Osu;
use serenity::model::channel::{Attachment, Message};
use serenity::model::id::{ChannelId, UserId};
use tokio::io::AsyncWriteExt;
use tokio::{
    fs::File,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

pub enum AttachmentParseResult {
    NoAttachmentOrReplay,
    BeingProcessed,
    FailedDownload(serenity::prelude::SerenityError),
    FailedParsing(osu_db::Error),
}

pub struct Data {
    path: String,
    replay: Replay,
    channel: ChannelId,
    user: UserId,
}

pub async fn process_replay(
    mut receiver: UnboundedReceiver<Data>,
    osu: Osu,
    http: Arc<CacheAndHttp>,
) {
    loop {
        let replay_data = receiver.recv().await.unwrap();

        let replay_path = replay_data.path;
        let replay_file = replay_data.replay;
        let replay_user = replay_data.user;
        let replay_channel = replay_data.channel;

        let hash = match replay_file.beatmap_hash {
            Some(h) => h,
            None => {
                println!("no hash in replay requested by user {}", replay_user);
                continue;
            }
        };

        let beatmap_info = osu.beatmap().checksum(hash).await;
        let map_id = match beatmap_info {
            Ok(map) => map.map_id,
            Err(why) => {
                println!("failed to request map with hash {}: {}", hash, why);
                continue;
            }
        };

        let download_result = prepare_beatmap_file(map_id).await;
        match download_result {
            Ok(path) => println!("download path: {}", path),
            Err(why) => {
                println!("failed to download: {}", why);
                continue;
            }
        }

        let settings: String = if fs::exists(format!("../danser/settings/{}.json", replay_user)) {
            format!("{}.json", replay_user);
        } else {
            "default.json";
        };

        let filename = replay_path
            .split('/')
            .last()
            .and_then(|file| file.split('.').next())
            .unwrap();

        let mut command = Command::new("../danser/danser");

        command
            .arg("-replay={}", replay_path)
            .arg("-record")
            .arg("-settings={}", settings)
            .arg("-out={}", filename);

        let output = command.output.await;

        // uploader::upload(format!("../Replays/{}.mp4", filename));
        replay_channel.send_message(&http, "uploaded").await;
    }
}

pub async fn parse_attachment_replay(
    msg: &Message,
    sender: &UnboundedSender<Data>,
) -> AttachmentParseResult {
    let attachment = match msg.attachments.last() {
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
                let mut file = File::create(format!("../Downloads/{}", &attachment.filename))
                    .await
                    .expect("failed to create file");

                file.write_all(&bytes).await.expect("failed to write");

                let replay_data = Data {
                    path: String::from(format!("../Downloads/{}", &attachment.filename)),
                    replay: replay,
                    channel: msg.channel_id,
                    user: msg.author.id,
                };

                if let Err(why) = sender.send(replay_data) {
                    println!("failed to send: {}", why);
                }
                return AttachmentParseResult::BeingProcessed;
            }
            Err(why) => AttachmentParseResult::FailedParsing(why),
        },
        Err(why) => AttachmentParseResult::FailedDownload(why),
    }
}
