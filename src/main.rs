mod schedule_data;
mod splatfest_data;
mod error;

extern crate serde;
extern crate chrono;

use std::{env, fmt::Display, fs, path::Path, future::Future};

use chrono::{DateTime, Utc};
use error::{Error, Result};
use reqwest::{Body, Client, IntoUrl, StatusCode};
use schedule_data::RotationData;
use serde::{de, Serialize};
use splatfest_data::SplatfestData;
use tokio::join;
use webhook::models::{Embed, Message};

const DISCORD_WEBHOOK_URL: &str = match cfg!(debug_assertions) {
    false => r#"https://discord.com/api/webhooks/1259694126161592341/4w7LylHC_vMTkX718KPcAncbMqUXh-ed4JHKjiMhtNVRvt8MXI_FjuK3Gwa1Pe2IXGug"#,// Release
    true => r#"https://discord.com/api/webhooks/1259137224432422974/H9LGZTTfEbeVw2Gng2f_SYHGd6CnZPJ1KM5a_mUfZpYjrwrVA0w53hdAp__0JGKcNXL6"#, // Testing
};
const SCHEDULES_URL: &str = r#"https://splatoon3.ink/data/schedules.json"#;
const SPLATFEST_URL: &str = r#"https://splatoon3.ink/data/festivals.json"#;

#[tokio::main]
async fn main() -> Result<()> {
    let reqwest_client = Client::builder()
        .user_agent(env!("CARGO_PKG_NAME"))
        .build()?
    ;
    let (schedules, splatfests) = join!(
        async {Ok::<_, Error>(send_notifications(&reqwest_client, &get_salmon_run_notifications(&reqwest_client).await?).await)},
        async {Ok::<_, Error>(send_notifications(&reqwest_client, &get_splatfest_notifications(&reqwest_client).await?).await)},
    );
    let (ok, err) = schedules?.into_iter().chain(splatfests?).partition::<Vec<_>,_>(|res| res.is_ok());
    println!("Notifs sent: {} | Notifs failed: {}", ok.len(), err.len());
    Ok(())
}

#[derive(Debug)]
enum Notification {
    Splatfest {
        title: String,
        teams: [String; 3],
        team_image: String,
        start: DateTime<Utc>,
        tricolor: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    BigRun {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        king: String,
        stage: (String, String),
    },
    EggstraWork {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        weapons: [String; 4],
        stage: (String, String),
    },
    Random {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        weapons: Vec<String>,
        king: String,
        stage: (String, String),
    },
    Golden {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        king: String,
        stage: (String, String),
    },
}

impl Notification {
    const THUMBNAIL_SPLATFEST: &'static str = r#"https://cdn.discordapp.com/attachments/842036323652337690/1259640933893275711/SfOpenSche.png?ex=668c6b89&is=668b1a09&hm=2cf5bd8276ae08f1791aa27590f2d4546166754df40ae8ee96e536f4a9a65769&"#;
    const THUMBNAIL_BIG_RUN: &'static str = r#"https://cdn.wikimg.net/en/splatoonwiki/images/7/73/S3_Badge_Big_Run_Top_50_Percent.png"#;
    const THUMBNAIL_EGGSTRA_WORK: &'static str = r#"https://cdn.wikimg.net/en/splatoonwiki/images/3/36/S3_Badge_Eggstra_Work_Top_5_Percent.png"#;
    const THUMBNAIL_RANDOM: &'static str = r#"https://splatoon3.ink/assets/splatnet/v2/ui_img/473fffb2442075078d8bb7125744905abdeae651b6a5b7453ae295582e45f7d1_0.png"#;
    const THUMBNAIL_GOLDEN: &'static str = r#"https://cdn.wikimg.net/en/splatoonwiki/images/7/73/S3_Badge_Big_Run_Top_50_Percent.png"#;
    fn thumbnail(&self) -> &'static str {
        match self {
            Notification::Splatfest{..} => Self::THUMBNAIL_SPLATFEST,
            Notification::BigRun{..} => Self::THUMBNAIL_BIG_RUN,
            Notification::EggstraWork{..} => Self::THUMBNAIL_EGGSTRA_WORK,
            Notification::Random{..} => Self::THUMBNAIL_RANDOM,
            Notification::Golden{..} => Self::THUMBNAIL_GOLDEN,
        }
    }

    const TITLE_SPLATFEST: &'static str = "A Splatfest has been announced!";
    const TITLE_BIG_RUN: &'static str = "A Big Run alert has been broadcasted!";
    const TITLE_EGGSTRA_WORK: &'static str = "Eggstra Workers are needed at Grizzco!";
    const TITLE_SINGLE_RANDOM: &'static str = "A Single Random Rotation has been added to the schedule!";
    const TITLE_PARTIAL_RANDOM: &'static str = "A Partial Random Rotation has been added to the schedule!";
    const TITLE_FULL_RANDOM: &'static str = "A Random Rotation has been added to the schedule!";
    const TITLE_GOLDEN: &'static str = "A Golden Rotation has been added to the schedule!";
    fn title(&self) -> &'static str {
        match self {
            Notification::Splatfest{..} => Self::TITLE_SPLATFEST,
            Notification::BigRun{..} => Self::TITLE_BIG_RUN,
            Notification::EggstraWork{..} => Self::TITLE_EGGSTRA_WORK,
            Notification::Random{weapons, ..} => {
                match weapons.len() {
                    0 | 1 => Self::TITLE_SINGLE_RANDOM,
                    2 | 3 => Self::TITLE_PARTIAL_RANDOM,
                    _ => Self::TITLE_FULL_RANDOM
                }
            },
            Notification::Golden{..} => Self::TITLE_GOLDEN,
        }
    }

    const COLOR_SPLATFEST: u32 = 0x2f5dd4;
    const COLOR_BIG_RUN: u32 = 0xB322FF;
    const COLOR_RANDOM: u32 = 0x00D82D;
    const COLOR_GOLDEN: u32 = 0xD18E14;
    fn color(&self) -> u32 {
        match self {
            Notification::Splatfest{..} => Self::COLOR_SPLATFEST,
            Notification::BigRun{..} => Self::COLOR_BIG_RUN,
            Notification::Random{..} => Self::COLOR_RANDOM,
            Notification::EggstraWork{..} |
            Notification::Golden{..} => Self::COLOR_GOLDEN,
        }
    }

    const PING_SPLATFEST: &'static str = match cfg!(debug_assertions) {
        false => r#"<@&1218339314057089136>"#,// Release
        true => r#"<@&1259331029781577830>"#, // Testing
    };
    const PING_SALMON_RUN: &'static str = match cfg!(debug_assertions) {
        false => r#"<@&1218339752659521568>"#,// Release
        true => r#"<@&842036705641234440>"#,  // Testing
    };
    
    fn ping(&self) -> &'static str {
        match self {
            Notification::Splatfest{..} => Self::PING_SPLATFEST,
            Notification::BigRun{..} |
            Notification::EggstraWork{..} |
            Notification::Random{..} |
            Notification::Golden{..} => Self::PING_SALMON_RUN,
        }
    }

    const AVATAR_SPLATFEST: &'static str = r#"https://cdn.discordapp.com/attachments/842036323652337690/1259640933893275711/SfOpenSche.png?ex=668c6b89&is=668b1a09&hm=2cf5bd8276ae08f1791aa27590f2d4546166754df40ae8ee96e536f4a9a65769&"#;
    const AVATAR_GRIZZCO: &'static str = r#"https://cdn.wikimg.net/en/splatoonwiki/images/8/8a/S3_Brand_Grizzco.png?20240224045446"#;
    fn avatar(&self) -> &'static str {
        match self {
            Notification::Splatfest{..} => Self::AVATAR_SPLATFEST,
            Notification::BigRun{..} |
            Notification::EggstraWork{..} |
            Notification::Random{..} |
            Notification::Golden{..} => Self::AVATAR_GRIZZCO,
        }
    }

    const NAME_SPLATFEST: &'static str = r#"Fax Machine"#;
    const NAME_GRIZZCO: &'static str = r#"Grizzco"#;
    fn name(&self) -> &'static str {
        match self {
            Notification::Splatfest{..} => Self::NAME_SPLATFEST,
            Notification::BigRun{..} |
            Notification::EggstraWork{..} |
            Notification::Random{..} |
            Notification::Golden{..} => Self::NAME_GRIZZCO,
        }
    }

    fn prefix_embed<'a>(&self, embed: &'a mut Embed) -> &'a mut Embed {
        embed
            .title(self.title())
            .color(self.color().to_string().as_str())
            .thumbnail(self.thumbnail())
    }

    fn setup_message<'a>(&'a self, message: &'a mut Message) -> &'a mut Message {
        message
            .content(self.ping())
            .avatar_url(self.avatar())
            .username(self.name())
        ;
        match self {
            Notification::Splatfest{title, teams, team_image, start, tricolor, end} => {
                let start_stamp = start.timestamp();
                let tricolor_stamp = tricolor.timestamp();
                let end_stamp = end.timestamp();
                message
                    .embed(|embed| self.prefix_embed(embed)
                        .field(&format!("Starts <t:{start_stamp}:R>"), &format!("<t:{start_stamp}:f>"), true)
                        .field(&format!("Tricolor <t:{tricolor_stamp}:R>"), &format!("<t:{tricolor_stamp}:f>"), true)
                        .field(&format!("Ends <t:{end_stamp}:R>"), &format!("<t:{end_stamp}:f>"), true)
                        .field(title, teams.into_iter().cloned().reduce(|acc, e| format!("{acc}\n{e}")).unwrap_or_default().as_str(), false)
                        .image(team_image)
                    )
                ;
            },
            Notification::EggstraWork{start, end, weapons, stage} => {
                let start_stamp = start.timestamp();
                let end_stamp = end.timestamp();
                message
                    .embed(|embed| self.prefix_embed(embed)
                        .field(&format!("Starts <t:{start_stamp}:R>"), &format!("<t:{start_stamp}:f>"), true)
                        .field(&format!("Ends <t:{end_stamp}:R>"), &format!("<t:{end_stamp}:f>"), true)
                        .field("Weapons", weapons.into_iter().cloned().reduce(|acc, e| format!("{acc}\n{e}")).unwrap_or_default().as_str(), false)
                        .field("Stage", &stage.0, false)
                        .image(&stage.1)
                    )
                ;
            },
            Notification::Random{start, end, weapons, king, stage} => {
                let start_stamp = start.timestamp();
                let end_stamp = end.timestamp();
                message
                    .embed(|embed| self.prefix_embed(embed)
                        .field(&format!("Starts <t:{start_stamp}:R>"), &format!("<t:{start_stamp}:f>"), true)
                        .field(&format!("Ends <t:{end_stamp}:R>"), &format!("<t:{end_stamp}:f>"), true)
                        .field("Weapons", weapons.into_iter().cloned().reduce(|acc, e| format!("{acc}\n{e}")).unwrap_or_default().as_str(), false)
                        .field("King Salmonid", king, false)
                        .field("Stage", &stage.0, false)
                        .image(&stage.1)
                    )
                ;
            },
            Notification::BigRun{start, end, king, stage} |
            Notification::Golden{start, end, king, stage} => {
                let start_stamp = start.timestamp();
                let end_stamp = end.timestamp();
                message
                    .embed(|embed| self.prefix_embed(embed)
                        .field(&format!("Starts <t:{start_stamp}:R>"), &format!("<t:{start_stamp}:f>"), true)
                        .field(&format!("Ends <t:{end_stamp}:R>"), &format!("<t:{end_stamp}:f>"), true)
                        .field("King Salmonid", king, false)
                        .field("Stage", &stage.0, false)
                        .image(&stage.1)
                        .image(&stage.1)
                    )
                ;
            },
        }
        message
    }
}

impl Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Notification::Splatfest{title, ..} => write!(f, "Splatfest: {title}"),
            Notification::BigRun{stage, ..} => write!(f, "Big Run on {}", stage.0),
            Notification::EggstraWork{stage, ..} => write!(f, "Eggstra Work on {}", stage.0),
            Notification::Random{stage, weapons, ..} => match weapons.len() {
                0 | 1 => write!(f, "Random Rotation on {}", stage.0),
                2 | 3 => write!(f, "Random Rotation on {}", stage.0),
                _ => write!(f, "Random Rotation on {}", stage.0),
            },
            Notification::Golden{stage, ..} => write!(f, "Golden Rotation on {}", stage.0),
        }
    }
}

async fn fetch_json<U: IntoUrl, T: de::DeserializeOwned>(reqwest_client: &Client, url: U) -> Result<T> {
    let json = reqwest_client
        .get(url)
        .send()
        .await?
        .text()
        .await?
    ;
    let data = serde_json::from_str(&json)?;
    Ok(data)
}
fn read_file<P: AsRef<Path>, T: de::DeserializeOwned>(path: P) -> Result<T> {
    let json = fs::read_to_string(path)?;
    let data = serde_json::from_str(&json)?;
    Ok(data)
}

async fn get_data<T, U, P>(reqwest_client: &Client, url: U, path: P) -> Result<(T, T)>
    where
        U: IntoUrl,
        P: AsRef<Path>,
        T: de::DeserializeOwned + Serialize + Clone,
    {
    let path = env::current_dir()?.join(path);
    let internet_data: T = match fetch_json(reqwest_client, url).await {
        Ok(data) => data,
        Err(Error::Reqwest(err)) if err.is_connect() => read_file(&path)?,
        Err(err) => return Err(err),
    };
    let file_data = if path.exists() {
        let file = read_file(&path)?;
        fs::write(&path, serde_json::to_string(&internet_data)?)?;
        file
    } else {
        fs::write(&path, serde_json::to_string(&internet_data)?)?;
        internet_data.clone()
    };
    Ok((internet_data, file_data))
}

async fn get_salmon_run_notifications(reqwest_client: &Client) -> Result<Vec<Notification>> {
    let (internet_data, file_data) = get_data::<RotationData,_,_>(reqwest_client, SCHEDULES_URL, "Schedules Json.json").await?;
    const RANDOM_WEAPON_ID: &str = "52e07029f01362a4";
    const GOLDEN_WEAPON_ID: &str = "obaiwjeobjo";
    let regular_notifications = 
        internet_data.data.coopGroupingSchedule.regularSchedules.nodes.into_iter().take_while(|internet_event|
            file_data.data.coopGroupingSchedule.regularSchedules.nodes.iter().all(|file_event| internet_event != file_event)
        )
        .filter_map(|event|
            event.setting.weapons.iter().any(|weapon| weapon.__splatoon3ink_id.contains(RANDOM_WEAPON_ID)).then(||
                Notification::Random {
                    start: event.startTime.to_utc(),
                    end: event.endTime.to_utc(),
                    weapons: event.setting.weapons.iter().cloned().map(|weapon| weapon.name).collect(),
                    king: event.__splatoon3ink_king_salmonid_guess.clone(),
                    stage: (
                        event.setting.coopStage.name.clone(),
                        event.setting.coopStage.image.url.clone()
                    )
                }
            )
            .or_else(|| event.setting.weapons.iter().any(|weapon| weapon.__splatoon3ink_id.contains(GOLDEN_WEAPON_ID)).then(|| 
                Notification::Golden {
                    start: event.startTime.to_utc(),
                    end: event.endTime.to_utc(),
                    king: event.__splatoon3ink_king_salmonid_guess,
                    stage: (
                        event.setting.coopStage.name,
                        event.setting.coopStage.image.url
                    )
                }
            ))
        )
    ;
    let big_run_notifications = 
        internet_data.data.coopGroupingSchedule.bigRunSchedules.nodes.into_iter().take_while(|internet_event|
            file_data.data.coopGroupingSchedule.bigRunSchedules.nodes.iter().all(|file_event| internet_event != file_event)
        )
        .map(|event|
            Notification::BigRun {
                start: event.startTime.to_utc(),
                end: event.endTime.to_utc(),
                king: event.__splatoon3ink_king_salmonid_guess,
                stage: (
                    event.setting.coopStage.name,
                    event.setting.coopStage.image.url
                )
            }
        )
    ;
    let eggstra_work_schedule = 
        internet_data.data.coopGroupingSchedule.teamContestSchedules.nodes.into_iter().take_while(|internet_event|
            file_data.data.coopGroupingSchedule.teamContestSchedules.nodes.iter().all(|file_event| internet_event != file_event))
        .map(|event| 
            Notification::EggstraWork {
                start: event.startTime.to_utc(),
                end: event.endTime.to_utc(),
                weapons: event.setting.weapons.map(|weapon| weapon.name),
                stage: (
                    event.setting.coopStage.name,
                    event.setting.coopStage.image.url,
                )
            }
        )
    ;
    Ok(regular_notifications.chain(big_run_notifications).chain(eggstra_work_schedule).collect())
}

async fn get_splatfest_notifications(reqwest_client: &Client) -> Result<Vec<Notification>> {
    let (internet_data, file_data) = get_data::<SplatfestData,_,_>(reqwest_client, SPLATFEST_URL, "Splatfest Json.json").await?;
    let splatfest_notifications = 
        internet_data.US.data.festRecords.nodes.into_iter().take_while(|internet_fest| file_data.US.data.festRecords.nodes.iter().all(|file_fest| internet_fest != file_fest))
        .map(|fest| 
            Notification::Splatfest {
                title: fest.title,
                teams: fest.teams.map(|team| team.teamName),
                team_image: fest.image.url,
                start: fest.startTime.to_utc(),
                tricolor: fest.startTime.to_utc() + ((fest.endTime - fest.startTime) / 2),
                end: fest.endTime.to_utc()
            }
        )
    ;
    Ok(splatfest_notifications.collect())
}

async fn send_notifications(reqwest_client: &Client, notifications: &[Notification]) -> Vec<Result<()>> {
    collect_futures(notifications.into_iter().map(|notif| async move {
        let mut message = Message::new();
        notif.setup_message(&mut message);
        println!("{notif}");
        loop {
            match send_message(&reqwest_client, &message).await {
                Ok(b) => break Ok(b),
                Err(Error::Discord(err)) => {
                    async_std::task::sleep(std::time::Duration::from_secs_f64(err.retry_after)).await;
                    continue;
                },
                Err(err) => break Err(err),
            }
        }
        .inspect_err(|err| eprintln!("Sending Err: {err}"))
    }))
    .await
}

async fn collect_futures<O, I>(iter: I) -> O
where
    I: IntoIterator,
    I::Item: Future,
    O: Default + Extend<<I::Item as Future>::Output>,
{
    let mut futures: Vec<_> = iter.into_iter().map(Box::pin).collect();
    let mut results = O::default();
    while !futures.is_empty() {
        let (res, _, remaining) = futures::future::select_all(futures).await;
        results.extend(std::iter::once(res));
        futures = remaining;
    }
    results
}

async fn send_message(reqwest_client: &Client, message: &Message) -> Result<()> {
    let body = serde_json::to_string(message)?;
    let response = reqwest_client
        .post(DISCORD_WEBHOOK_URL)
        .header("content-type", "application/json")
        .body(Body::from(body))
        .send()
        .await?
    ;
    if response.status() == StatusCode::NO_CONTENT {
        Ok(())
    } else {
        let body_bytes = response.bytes().await?;
        let err_msg = String::from_utf8(body_bytes.to_vec())?;
        Err(Error::Discord(serde_json::from_str(&err_msg)?))
    }
}
