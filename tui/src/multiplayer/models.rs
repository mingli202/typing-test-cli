use std::collections::HashMap;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::util::data_provider::Data;

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct PlayerInfo {
    pub is_leader: bool,
    pub wpm: f64,
    pub progress_percent: u8,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PlayerInfoSnapshot {
    pub version: u64,
    pub players: HashMap<String, PlayerInfo>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct LobbyInfo {
    pub lobby_id: String,
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct NewGame {
    pub data: Data,
    pub players_info: PlayerInfoSnapshot,
}

#[derive(Debug)]
pub enum WsMsg {
    NewGroup,
    JoinGroup(String),
    LeaveGroup,
    UpdateStats(f64, u8),
    StartGame,
}

impl Display for WsMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            WsMsg::JoinGroup(group_id) => format!("JoinGroup {}", group_id),
            WsMsg::UpdateStats(wpm, progress) => format!("UpdateStats {:.1} {}", wpm, progress),
            _ => format!("{:?}", self),
        };

        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod test {
    use super::WsMsg;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_wsmsg_to_string_1() {
        let msg = WsMsg::NewGroup;

        assert_eq!("NewGroup".to_string(), msg.to_string())
    }

    #[test]
    fn test_wsmsg_to_string_2() {
        let msg = WsMsg::JoinGroup("asdfgh".to_string());

        assert_eq!("JoinGroup asdfgh".to_string(), msg.to_string())
    }

    #[test]
    fn test_wsmsg_to_string_3() {
        let msg = WsMsg::UpdateStats(10.123, 100);

        assert_eq!("UpdateStats 10.1 100".to_string(), msg.to_string())
    }
}
