use crate::types::{
    AnalyzeHandRequestV1, MeldInput, MeldKind, RiichiInput, ScoreBreakdown, ScoringResultV1,
    SpecialWinInput, TsumoPoints, WinMethodInput, WindInput, YakuResult,
};
use riichi_calc::calculator::result::Points;
use riichi_calc::calculator::score::calc_score;
use riichi_calc::constants::field::{Field, Wind};
use riichi_calc::constants::hand::Mentsu;
use riichi_calc::constants::status::{RiichiStatus, SpecialWin, Status, WinMethod};
use riichi_calc::constants::tiles::{Tile, TileType};
use riichi_calc::finder::finder::Finder;
use riichi_calc::finder::result::{FoundResult, FoundYaku, FoundYakuman};
use riichi_calc::finder::yaku::YakuEntry;
use riichi_calc::parser::{Input, PiInput};
use std::collections::HashSet;

pub fn analyze_hand(request: AnalyzeHandRequestV1) -> ScoringResultV1 {
    match analyze_hand_inner(request) {
        Ok(result) => result,
        Err(error) => ScoringResultV1 {
            is_win: false,
            yaku: Vec::new(),
            han: 0,
            fu: 0,
            score_breakdown: None,
            waits: Vec::new(),
            errors: vec![error],
        },
    }
}

fn analyze_hand_inner(request: AnalyzeHandRequestV1) -> Result<ScoringResultV1, String> {
    let winning_tile_id = request
        .winning_tile
        .clone()
        .ok_or_else(|| "请先标记和了牌".to_string())?;
    let winning_tile = tile_from_id(&winning_tile_id)?;
    let hand_tiles =
        normalize_hand_tiles(&request.hand_tiles, &winning_tile_id, request.melds.len())?;
    let hand = hand_tiles
        .iter()
        .map(|tile| tile_from_id(tile))
        .collect::<Result<Vec<_>, _>>()?;
    let naki = request
        .melds
        .iter()
        .map(meld_to_mentsu)
        .collect::<Result<Vec<_>, _>>()?;
    let dora = request
        .dora_indicators
        .iter()
        .map(|tile| dora_from_indicator(tile))
        .collect::<Result<Vec<_>, _>>()?;
    let ura_dora = request
        .ura_dora_indicators
        .iter()
        .map(|tile| dora_from_indicator(tile))
        .collect::<Result<Vec<_>, _>>()?;

    let field = Field {
        zikaze: wind_from_input(&request.self_wind),
        bakaze: wind_from_input(&request.round_wind),
        honba: request.honba,
        dora,
    };
    let status = Status {
        riichi: riichi_from_input(&request.riichi, ura_dora),
        win_method: win_method_from_input(&request.win_method),
        special_win: special_wins(&request.special_win),
    };
    let input = Input::new(
        PiInput {
            hand,
            naki,
            hora: winning_tile,
        },
        field.clone(),
        status.clone(),
    );

    let parsed = input
        .parse_hand()
        .map_err(|err| format!("输入不是合法和牌：{err:?}"))?;
    let mut best: Option<ScoringResultV1> = None;

    for hand in parsed {
        let found = Finder::find_hand(&hand);
        if !found.is_valid_hora() {
            continue;
        }
        let score = calc_score(&found, &field, &hand.winning_hand, &status);
        let points = corrected_actual_points(
            score.detail.fu,
            score.detail.han,
            &found,
            &status.win_method,
            request.self_wind == WindInput::East,
            request.honba,
        );
        let is_yakuman = matches!(found, FoundResult::FoundYakuman(_));
        let candidate = ScoringResultV1 {
            is_win: true,
            yaku: flatten_yaku(&found),
            han: score.detail.han,
            fu: score.detail.fu,
            score_breakdown: Some(score_breakdown(
                &points,
                score.detail.fu,
                score.detail.han,
                is_yakuman,
            )),
            waits: Vec::new(),
            errors: Vec::new(),
        };

        if best
            .as_ref()
            .map(|current| point_rank(current) < point_rank(&candidate))
            .unwrap_or(true)
        {
            best = Some(candidate);
        }
    }

    best.ok_or_else(|| "没有找到役种，无法和牌".to_string())
}

fn normalize_hand_tiles(
    hand_tiles: &[String],
    winning_tile: &str,
    meld_count: usize,
) -> Result<Vec<String>, String> {
    let expected = 13usize.saturating_sub(meld_count * 3);
    if hand_tiles.len() == expected {
        return Ok(hand_tiles.to_vec());
    }
    if hand_tiles.len() == expected + 1 {
        let mut normalized = hand_tiles.to_vec();
        if let Some(index) = normalized.iter().position(|tile| tile == winning_tile) {
            normalized.remove(index);
            return Ok(normalized);
        }
    }
    Err(format!(
        "手牌数量不正确：当前 {} 张，副露 {} 组时应为 {} 张（不含和了牌）",
        hand_tiles.len(),
        meld_count,
        expected
    ))
}

fn tile_from_id(tile_id: &str) -> Result<Tile, String> {
    if tile_id.starts_with('f') {
        return Err(format!("{tile_id} 是花牌/季节牌，v1 不参与日麻算番"));
    }

    let bytes = tile_id.as_bytes();
    if bytes.len() < 2 {
        return Err(format!("未知牌 ID：{tile_id}"));
    }

    let tile_type = match bytes[0] as char {
        'm' => TileType::Manzu,
        'p' => TileType::Pinzu,
        's' => TileType::Souzu,
        'z' => {
            let number = tile_id[1..]
                .parse::<u8>()
                .map_err(|_| format!("未知字牌：{tile_id}"))?;
            return match number {
                1..=4 => Ok(Tile {
                    number,
                    tile_type: TileType::Wind,
                }),
                5..=7 => Ok(Tile {
                    number: number - 4,
                    tile_type: TileType::Dragon,
                }),
                _ => Err(format!("未知字牌：{tile_id}")),
            };
        }
        _ => return Err(format!("未知牌 ID：{tile_id}")),
    };

    let number = if tile_id.ends_with('r') {
        10
    } else {
        tile_id[1..]
            .parse::<u8>()
            .map_err(|_| format!("未知数牌：{tile_id}"))?
    };
    if !(1..=10).contains(&number) {
        return Err(format!("未知数牌：{tile_id}"));
    }

    Ok(Tile { number, tile_type })
}

fn meld_to_mentsu(meld: &MeldInput) -> Result<Mentsu, String> {
    if meld.tiles.is_empty() {
        return Err("副露不能为空".to_string());
    }
    let mut tiles = meld
        .tiles
        .iter()
        .map(|tile| tile_from_id(tile).map(normalize_red))
        .collect::<Result<Vec<_>, _>>()?;

    match meld.kind {
        MeldKind::Chi => {
            if tiles.len() != 3 {
                return Err("吃必须正好 3 张牌".to_string());
            }
            tiles.sort_by_key(|tile| tile.number);
            if tiles
                .iter()
                .any(|tile| tile.tile_type != tiles[0].tile_type)
                || matches!(tiles[0].tile_type, TileType::Wind | TileType::Dragon)
                || tiles[1].number != tiles[0].number + 1
                || tiles[2].number != tiles[1].number + 1
            {
                return Err("吃必须是同花色连续三张数牌".to_string());
            }
            Ok(Mentsu::Shuntsu(tiles[0], true))
        }
        MeldKind::Pon => same_tile_mentsu(&tiles, 3).map(|tile| Mentsu::Koutsu(tile, true)),
        MeldKind::Daiminkan => same_tile_mentsu(&tiles, 4).map(|tile| Mentsu::Kantsu(tile, true)),
        MeldKind::Ankan => same_tile_mentsu(&tiles, 4).map(|tile| Mentsu::Kantsu(tile, false)),
    }
}

fn same_tile_mentsu(tiles: &[Tile], count: usize) -> Result<Tile, String> {
    if tiles.len() != count {
        return Err(format!("该副露必须正好 {count} 张牌"));
    }
    let first = tiles[0];
    if tiles.iter().all(|tile| *tile == first) {
        Ok(first)
    } else {
        Err("碰/杠必须由相同牌组成".to_string())
    }
}

fn normalize_red(tile: Tile) -> Tile {
    if tile.number == 10 {
        Tile {
            number: 5,
            tile_type: tile.tile_type,
        }
    } else {
        tile
    }
}

fn dora_from_indicator(tile_id: &str) -> Result<Tile, String> {
    let tile = normalize_red(tile_from_id(tile_id)?);
    let number = match tile.tile_type {
        TileType::Manzu | TileType::Pinzu | TileType::Souzu => {
            if tile.number == 9 {
                1
            } else {
                tile.number + 1
            }
        }
        TileType::Wind => {
            if tile.number == 4 {
                1
            } else {
                tile.number + 1
            }
        }
        TileType::Dragon => match tile.number {
            1 => 2,
            2 => 3,
            3 => 1,
            _ => return Err(format!("非法三元牌指示牌：{tile_id}")),
        },
    };
    Ok(Tile {
        number,
        tile_type: tile.tile_type,
    })
}

fn wind_from_input(wind: &WindInput) -> Wind {
    match wind {
        WindInput::East => Wind::East,
        WindInput::South => Wind::South,
        WindInput::West => Wind::West,
        WindInput::North => Wind::North,
    }
}

fn win_method_from_input(method: &WinMethodInput) -> WinMethod {
    match method {
        WinMethodInput::Ron => WinMethod::Ron,
        WinMethodInput::Tsumo => WinMethod::Tumo,
    }
}

fn riichi_from_input(riichi: &RiichiInput, ura_dora: Vec<Tile>) -> RiichiStatus {
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

fn flatten_yaku(found: &FoundResult) -> Vec<YakuResult> {
    let mut yaku = Vec::new();
    match found {
        FoundResult::FoundYaku(FoundYaku {
            dora,
            ii_han,
            ryan_han,
            san_han,
            roku_han,
        }) => {
            push_yaku(&mut yaku, dora, "dora");
            push_yaku(&mut yaku, ii_han, "one_han");
            push_yaku(&mut yaku, ryan_han, "two_han");
            push_yaku(&mut yaku, san_han, "three_han");
            push_yaku(&mut yaku, roku_han, "six_han");
        }
        FoundResult::FoundYakuman(FoundYakuman { yakuman }) => {
            push_yaku(&mut yaku, yakuman, "yakuman");
        }
    }
    yaku
}

fn push_yaku(target: &mut Vec<YakuResult>, source: &[YakuEntry], prefix: &str) {
    for entry in source {
        target.push(YakuResult {
            id: format!("{prefix}_{}", target.len() + 1),
            name: entry.name().to_string(),
            han: entry.value,
        });
    }
}

fn corrected_actual_points(
    fu: u8,
    han: u8,
    found: &FoundResult,
    win_method: &WinMethod,
    is_dealer: bool,
    honba: u8,
) -> Points {
    let base = base_points(fu, han, matches!(found, FoundResult::FoundYakuman(_)));
    match (is_dealer, win_method) {
        (true, WinMethod::Ron) => Points::Ron(round_points(base * 6) + honba as u32 * 300),
        (false, WinMethod::Ron) => Points::Ron(round_points(base * 4) + honba as u32 * 300),
        (true, WinMethod::Tumo) => Points::DealerTumo(round_points(base * 2) + honba as u32 * 100),
        (false, WinMethod::Tumo) => Points::ChildTumo(
            round_points(base) + honba as u32 * 100,
            round_points(base * 2) + honba as u32 * 100,
        ),
    }
}

fn base_points(fu: u8, han: u8, is_yakuman: bool) -> u32 {
    if is_yakuman {
        return 8000 * han as u32;
    }
    match han {
        0..=4 => (fu as u32 * 2_u32.pow(han as u32 + 2)).min(2000),
        5 => 2000,
        6 | 7 => 3000,
        8..=10 => 4000,
        11 | 12 => 6000,
        _ => 8000,
    }
}

fn round_points(points: u32) -> u32 {
    points.div_ceil(100) * 100
}

fn score_breakdown(points: &Points, fu: u8, han: u8, is_yakuman: bool) -> ScoreBreakdown {
    ScoreBreakdown {
        base_points: base_points(fu, han, is_yakuman),
        ron_points: match points {
            Points::Ron(value) => *value,
            _ => 0,
        },
        tsumo_points: match points {
            Points::ChildTumo(non_dealer, dealer) => Some(TsumoPoints {
                dealer: *dealer,
                non_dealer: *non_dealer,
            }),
            Points::DealerTumo(value) => Some(TsumoPoints {
                dealer: *value,
                non_dealer: *value,
            }),
            Points::Ron(_) => None,
        },
    }
}

fn point_rank(result: &ScoringResultV1) -> u32 {
    let Some(score) = &result.score_breakdown else {
        return 0;
    };
    if score.ron_points > 0 {
        return score.ron_points;
    }
    score
        .tsumo_points
        .as_ref()
        .map(|tsumo| tsumo.dealer + tsumo.non_dealer * 2)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_red_five_to_riichi_calc_red_number() {
        let tile = tile_from_id("m5r").unwrap();
        assert_eq!(tile.number, 10);
        assert_eq!(tile.tile_type, TileType::Manzu);
    }

    #[test]
    fn maps_honor_tiles() {
        assert_eq!(tile_from_id("z1").unwrap().tile_type, TileType::Wind);
        assert_eq!(tile_from_id("z5").unwrap().tile_type, TileType::Dragon);
        assert_eq!(tile_from_id("z5").unwrap().number, 1);
    }

    #[test]
    fn converts_dora_indicators() {
        assert_eq!(dora_from_indicator("m9").unwrap().number, 1);
        assert_eq!(dora_from_indicator("z4").unwrap().number, 1);
        assert_eq!(dora_from_indicator("z7").unwrap().number, 1);
    }

    #[test]
    fn rejects_flower_tiles_for_scoring() {
        assert!(tile_from_id("f1").is_err());
    }

    #[test]
    fn scores_simple_pinfu_riichi_ron() {
        let result = analyze_hand(AnalyzeHandRequestV1 {
            hand_tiles: vec![
                "m1", "m2", "m3", "m5", "m6", "m7", "p2", "p3", "p4", "s6", "s7", "s9", "s9",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            winning_tile: Some("s5".to_string()),
            melds: Vec::new(),
            self_wind: WindInput::East,
            round_wind: WindInput::East,
            win_method: WinMethodInput::Ron,
            riichi: RiichiInput::Riichi,
            special_win: SpecialWinInput::default(),
            honba: 0,
            dora_indicators: Vec::new(),
            ura_dora_indicators: Vec::new(),
        });

        assert!(result.is_win, "{:?}", result.errors);
        assert!(result.han >= 1);
        assert!(result.fu >= 20);
    }
}
