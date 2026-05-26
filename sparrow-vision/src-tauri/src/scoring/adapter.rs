use crate::types::{AnalyzeHandRequest, RiichiInput, SpecialWinInput, WinMethodInput, WindInput};
use riichi_calc::constants::field::{Field, Wind};
use riichi_calc::constants::hand::Mentsu;
use riichi_calc::constants::status::{RiichiStatus, SpecialWin, Status, WinMethod};
use riichi_calc::constants::tiles::Tile;
use riichi_calc::parser::{Input, PiInput};
use std::collections::HashSet;

pub fn input(hand: Vec<Tile>, naki: Vec<Mentsu>, hora: Tile, field: Field, status: Status) -> Input {
  Input::new(PiInput { hand, naki, hora }, field, status)
}

pub fn field(request: &AnalyzeHandRequest, dora: Vec<Tile>) -> Field {
  Field {
    zikaze: wind(&request.self_wind),
    bakaze: wind(&request.round_wind),
    honba: request.honba,
    dora,
  }
}

pub fn status(request: &AnalyzeHandRequest, ura_dora: Vec<Tile>) -> Status {
  Status {
    riichi: riichi(&request.riichi, ura_dora),
    win_method: win_method(&request.win_method),
    special_win: special_wins(&request.special_win),
  }
}

fn wind(wind: &WindInput) -> Wind {
  match wind {
    WindInput::East => Wind::East,
    WindInput::South => Wind::South,
    WindInput::West => Wind::West,
    WindInput::North => Wind::North,
  }
}

fn win_method(method: &WinMethodInput) -> WinMethod {
  match method {
    WinMethodInput::Ron => WinMethod::Ron,
    WinMethodInput::Tsumo => WinMethod::Tumo,
  }
}

fn riichi(riichi: &RiichiInput, ura_dora: Vec<Tile>) -> RiichiStatus {
  match riichi {
    RiichiInput::None => RiichiStatus::NoRiichi,
    RiichiInput::Riichi => RiichiStatus::Riichi(ura_dora),
    RiichiInput::DoubleRiichi => RiichiStatus::DoubleRiichi(ura_dora),
  }
}

fn special_wins(input: &SpecialWinInput) -> HashSet<SpecialWin> {
  let mut wins = HashSet::new();
  if input.ippatsu {
    wins.insert(SpecialWin::Ipatu);
  }
  if input.chankan {
    wins.insert(SpecialWin::Chankan);
  }
  if input.rinshan {
    wins.insert(SpecialWin::Rinshan);
  }
  if input.haitei {
    wins.insert(SpecialWin::Haitei);
  }
  if input.hotei {
    wins.insert(SpecialWin::Hotei);
  }
  if input.first_turn_tsumo {
    wins.insert(SpecialWin::DaiichiTumo);
  }
  wins
}
