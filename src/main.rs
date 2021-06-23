use std::collections::HashMap;

fn main() {}

fn cross(a: &str, b: &str) -> Vec<String> {
    let mut result = Vec::new();
    for i in a.chars() {
        for j in b.chars() {
            result.push(vec![i, j].into_iter().collect());
        }
    }

    result
}

const DIGITS: &str = "123456789";
const COLS: &str = DIGITS;
const ROWS: &str = "ABCDEFGHI";

// TODO: USE THIS?
type Units = Vec<String>;
type UnitsList = Vec<Units>;
type UnitsForSquare = HashMap<String, UnitsList>;
type PeersForSquare = HashMap<String, Units>;

#[derive(Debug)]
struct State {
    squares: Vec<String>,
    unitlist: UnitsList,
    units: UnitsForSquare,
    peers: PeersForSquare,
}

fn init_stuff() -> State {
    let squares = cross(ROWS, COLS);

    // unitlist
    let mut unitlist: UnitsList = vec![];
    for c in COLS.chars() {
        unitlist.push(cross(ROWS, &c.to_string()));
    }
    for r in ROWS.chars() {
        unitlist.push(cross(&r.to_string(), COLS));
    }
    for rs in vec!["ABC", "DEF", "GHI"] {
        for cs in vec!["123", "456", "789"] {
            unitlist.push(cross(rs, cs));
        }
    }

    // units
    let mut units: UnitsForSquare = HashMap::new();

    for s in &squares {
        for u in &unitlist {
            if u.contains(s) {
                let mut new_units = units.get(s).cloned().unwrap_or(vec![]);
                new_units.push(u.to_vec());
                units.insert(s.to_string(), new_units);
            }
        }
    }

    //              for s in squares)
    let mut peers = PeersForSquare::new();
    for s in &squares {
        // flatten
        let mut new_units: Units = units
            .get(s)
            .cloned()
            .unwrap()
            .into_iter()
            .flatten()
            .collect();

        new_units.sort();
        new_units.dedup();
        new_units.retain(|square| square != s);
        peers.insert(s.to_string(), new_units);
    }

    // done!
    State {
        squares: squares,
        unitlist: unitlist,
        peers: peers,
        units: units,
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Tests
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_cross() {
        assert_eq!(cross("AB", "12"), ["A1", "A2", "B1", "B2"]);
    }

    #[test]
    fn test_base() {
        let game = init_stuff();

        dbg!(&game);
        assert_eq!(game.squares.len(), 81);
        assert_eq!(game.unitlist.len(), 27);
        for s in game.squares {
            assert_eq!(game.units[&s].len(), 3);
            assert_eq!(game.peers[&s].len(), 20);
        }
        assert_eq!(
            game.units["C2"],
            [
                ["A2", "B2", "C2", "D2", "E2", "F2", "G2", "H2", "I2"],
                ["C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9"],
                ["A1", "A2", "A3", "B1", "B2", "B3", "C1", "C2", "C3"]
            ]
        );
        assert_eq!(
            game.peers["C2"],
            [
                "A1", "A2", "A3", "B1", "B2", "B3", "C1", "C3", "C4", "C5", "C6", "C7", "C8", "C9",
                "D2", "E2", "F2", "G2", "H2", "I2"
            ]
        )
    }
}
