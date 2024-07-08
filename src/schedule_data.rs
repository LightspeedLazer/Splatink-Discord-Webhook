#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused)]

use std::fmt::{Display, Debug};

use crate::splatfest_data::Color;

use super::serde::{Deserialize, Serialize};
use super::chrono::prelude::*;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct image {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RotationData {
    pub data: data,
}
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct data {
    pub regularSchedules: nodes<regularSchedule>,
    pub bankaraSchedules: nodes<bankaraSchedule>,
    pub xSchedules: nodes<xSchedule>,
    pub eventSchedules: nodes<eventSchedule>,
    pub festSchedules: nodes<festSchedule>,
    pub coopGroupingSchedule: coopGroupingSchedule,
    pub currentFest: Option<currentFest>,
    pub currentPlayer: Player,
    pub vsStages: nodes<vsStageRecon>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct nodes<T> {
    pub nodes: Vec<T>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct regularSchedule {
    pub startTime: DateTime<Local>,
    pub endTime: DateTime<Local>,
    pub regularMatchSetting: Option<regularMatchSetting>,
    pub festMatchSetting: Option<[festMatchSettingsFake;2]>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct regularMatchSetting {
    pub __isVsSetting: String,
    pub __typename: String,
    pub vsStages: [vsStage;2],
    pub vsRule: vsRule,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct vsStage {
    pub vsStageId: isize,
    pub name: String,
    pub image: image,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct vsRule {
    pub name: String,
    pub rule: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct festMatchSettingsFake {
    pub __typename: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct bankaraSchedule {
    pub startTime: DateTime<Local>,
    pub endTime: DateTime<Local>,
    pub bankaraMatchSettings: Option<(bankaraMatchSetting, bankaraMatchSetting)>,
    pub festMatchSetting: Option<[festMatchSettingsFake;2]>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct bankaraMatchSetting {
    pub __isVsSetting: String,
    pub __typename: String,
    pub vsStages: [vsStage;2],
    pub vsRule: vsRule,
    pub bankaraMode: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct xSchedule {
    pub startTime: DateTime<Local>,
    pub endTime: DateTime<Local>,
    pub xMatchSetting: Option<xMatchSetting>,
    pub festMatchSetting: Option<[festMatchSettingsFake;2]>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct xMatchSetting {
    pub __isVsSetting: String,
    pub __typename: String,
    pub vsStages: [vsStage;2],
    pub vsRule: vsRule,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct eventSchedule {
    pub leagueMatchSetting: leagueMatchSetting,
    pub timePeriods: Vec<timePeriod>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct timePeriod {
    pub startTime: DateTime<Local>,
    pub endTime: DateTime<Local>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct leagueMatchSetting {
    pub leagueMatchEvent: leagueMatchEvent,
    pub vsStages: [vsStage;2],
    pub __isVsSetting: String,
    pub __typename: String,
    pub vsRule: Option<vsRule>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct leagueMatchEvent {
    pub leagueMatchEventId: String,
    pub name: String,
    pub desc: String,
    pub regulationUrl: Option<String>,
    pub regulation: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct festSchedule {
    pub startTime: DateTime<Local>,
    pub festMatchSettings: Option<(festMatchSetting,festMatchSetting)>,
    pub endTime: DateTime<Local>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct festMatchSetting {
    pub __typename: String,
    pub __isVsSetting: String,
    pub vsStages: [vsStage;2],
    pub vsRule: vsRule,
    pub festMode: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct coopGroupingSchedule {
    pub bannerImage: Option<image>,
    pub regularSchedules: nodes<salmonRunRotation>,
    pub bigRunSchedules: nodes<bigRunRotation>,
    pub teamContestSchedules: nodes<eggstraWorkRotation>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct salmonRunRotation {
    pub startTime: DateTime<Local>,
    pub endTime: DateTime<Local>,
    pub setting: salmonRunSetting,
    pub __splatoon3ink_king_salmonid_guess: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct salmonRunSetting {
    pub __typename: String,
    pub coopStage: coopStage,
    pub __isCoopSetting: String,
    pub weapons: [weapon;4],
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct coopStage {
    pub name: String,
    pub thumbnailImage: image,
    pub image: image,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct weapon {
    pub __splatoon3ink_id: String,
    pub name: String,
    pub image: image,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct bigRunRotation {
    pub startTime: DateTime<Local>,
    pub endTime: DateTime<Local>,
    pub setting: salmonRunSetting,
    pub __splatoon3ink_king_salmonid_guess: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct eggstraWorkRotation {
    pub startTime: DateTime<Local>,
    pub endTime: DateTime<Local>,
    pub setting: salmonRunSetting,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct currentFest {
    pub id: String,
    pub title: String,
    pub startTime: DateTime<Local>,
    pub endTime: DateTime<Local>,
    pub midtermTime: DateTime<Local>,
    pub state: String,
    pub teams: [team;3],
    pub tricolorStage: tricolorStage,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct team {
    pub id: String,
    pub color: Color,
    pub myVoteState: Option<()>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct tricolorStage {
    pub name: String,
    pub image: image,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Player {
    pub userIcon: image,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct vsStageRecon {
    pub vsStageId: isize,
    pub originalImage: image,
    pub name: String,
    pub stats: Option<()>,
    pub id: String,
}