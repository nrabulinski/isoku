use crate::{
    packets::OsuEncode,
    r#match::{ScoringType, SlotStatus, Team, TeamType},
};

mod create;
pub use create::handle as create;
mod change_settings;
use crate::event_data;
pub use change_settings::handle as change_settings;

fn get_slot_ids(data: &[u8], slot_statuses: &&[SlotStatus; 16]) -> Result<([i32; 16], usize), ()> {
    let slot_statuses = *slot_statuses;
    let mut off = 0;
    let mut slot_ids = [-1i32; 16];
    for (i, s) in slot_statuses.iter().enumerate() {
        if !s.is_occupied() {
            continue;
        }
        let (&id, o) = i32::decode(&data[off..])?;
        off += o;
        slot_ids[i] = id;
    }
    Ok((slot_ids, off))
}

fn get_slot_mods<'a>(data: &'a [u8], freemod: &bool) -> Result<(Option<&'a [i32; 16]>, usize), ()> {
    if *freemod {
        let (res, off) = <[i32; 16]>::decode(data)?;
        Ok((Some(res), off))
    } else {
        Ok((None, 0))
    }
}

fn parse_name<'a>(data: &'a [u8]) -> Result<(&'a str, usize), ()> {
    let (name, off) = str::decode(data)?;
    let name = name.trim();
    if name.is_empty() {
        Err(())
    } else {
        Ok((name, off))
    }
}

#[event_data]
#[derive(Debug)]
struct MatchSettings {
    id: u16,
    in_progress: bool,
    match_type: u8,
    mods: u32,
    #[decoder(parse_name)]
    name: &str,
    password: &str,
    beatmap_name: &str,
    beatmap_id: u32,
    beatmap_md5: &str,
    slot_statuses: &[SlotStatus; 16],
    slot_teams: &[Team; 16],
    #[decoder(get_slot_ids: slot_statuses)]
    slot_ids: [i32; 16],
    host_id: i32,
    game_mode: u8,
    scoring_type: ScoringType,
    team_type: TeamType,
    freemod: bool,
    #[decoder(get_slot_mods: freemod)]
    slot_mods: Option<&[i32; 16]>,
    seed: i32,
}
