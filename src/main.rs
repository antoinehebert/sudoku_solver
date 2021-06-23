use std::collections::HashMap;

fn main() {
    dbg!(parse_grid(&GRID2, &init_stuff()));
}

fn cross(a: &Vec<String>, b: &Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for i in a {
        for j in b {
            let mut new_str: String = i.clone();
            new_str.push_str(j);
            result.push(new_str);
        }
    }

    result
}

// TODO: USE THIS?
type Square = String;
type Units = Vec<String>;
type UnitsList = Vec<Units>;
type UnitlistsForSquare = HashMap<String, UnitsList>;
type UnitsForSquare = HashMap<String, Units>;

#[derive(Debug)]
struct State {
    digits: Vec<String>,
    cols: Vec<String>,
    rows: Vec<String>,
    squares: Vec<String>,
    unitlist: UnitsList,
    units: UnitlistsForSquare,
    peers: UnitsForSquare,
}

fn string_to_vec(s: &str) -> Vec<String> {
    s.chars().map(|s| s.to_string()).collect()
}

fn init_stuff() -> State {
    let digits: Vec<String> = string_to_vec("123456789");
    let cols: Vec<String> = string_to_vec("123456789");
    let rows: Vec<String> = string_to_vec("ABCDEFGHI");

    let squares = cross(&rows, &cols);

    // unitlist
    let mut unitlist: UnitsList = vec![];
    for c in &cols {
        unitlist.push(cross(&rows, &vec![c.clone()]));
    }
    for r in &rows {
        unitlist.push(cross(&vec![r.clone()], &cols));
    }
    for rs in vec![
        string_to_vec("ABC"),
        string_to_vec("DEF"),
        string_to_vec("GHI"),
    ] {
        for cs in vec![
            string_to_vec("123"),
            string_to_vec("456"),
            string_to_vec("789"),
        ] {
            unitlist.push(cross(&rs, &cs));
        }
    }

    // units
    let mut units: UnitlistsForSquare = HashMap::new();

    for s in &squares {
        for u in &unitlist {
            if u.contains(s) {
                units
                    .entry(s.clone())
                    .or_insert_with(Vec::new)
                    .push(u.clone());
            }
        }
    }

    //              for s in squares)
    let mut peers = UnitsForSquare::new();
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
        // DRY
        digits: digits,
        cols: cols,
        rows: rows,
        squares: squares,
        unitlist: unitlist,
        peers: peers,
        units: units,
    }
}

fn parse_grid(grid: &str, state: &State) -> Option<UnitsForSquare> {
    if grid.len() != 81 {
        return None;
    }

    // Start with all possible values
    let mut result = UnitsForSquare::new();

    for (index, square) in state.squares.iter().enumerate() {
        let digit = &grid.chars().nth(index).unwrap().to_string();

        if state.digits.contains(digit) {
            assign(&mut result, square, digit, state);
        } else {
            result.insert(square.clone(), state.digits.clone());
        }
    }

    Some(result)
}

fn assign(grid: &mut UnitsForSquare, square: &Square, digit: &String, state: &State) {
    let mut other_digits = grid[square].clone();
    other_digits.retain(|d| d != digit);
    for d in &other_digits {
        eliminate(grid, square, d, state);
    }
}

fn eliminate(grid: &mut UnitsForSquare, square: &Square, digit: &String, state: &State) {
    // Eliminate!
    if !grid[square].contains(digit) {
        return; // Already eliminated.
    }

    grid.get_mut(square).unwrap().retain(|d| d != digit);

    if grid[square].len() < 1 {
        // TODO: Propagate errors.
        println!("Contradiction. Removed the last digit from {}", square);
        return;
    } else if grid[square].len() == 1 {
        // Found a match, eliminate from peers.
        let d = &grid[square][0].clone();
        for s2 in &state.peers[square] {
            eliminate(grid, s2, d, state);
        }
    }

    // If a unit is reduced to only one place for a value, then put it there.
    for unit in &state.units[square] {
        let mut dplaces = Vec::new();
        for s in unit {
            if grid[s].contains(digit) {
                dplaces.push(s);
            }
        }
        if dplaces.len() == 0 {
            println!("Contradiction, no place for {}", digit);
            return;
        } else if dplaces.len() == 1 {
            // Digit can only be in one place in unit, assign it here.
            assign(grid, dplaces[0], digit, state);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Tests
////////////////////////////////////////////////////////////////////////////////

const GRID1: &str =
    "003020600900305001001806400008102900700000008006708200002609500800203009005010300";
const GRID2: &str =
    "4.....8.5.3..........7......2.....6.....8.4......1.......6.3.7.5..2.....1.4......";
const HARD_GRID1: &str =
    ".....6....59.....82....8....45........3........6..3.54...325..6..................";

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_cross() {
        assert_eq!(
            cross(&string_to_vec("AB"), &string_to_vec("12")),
            ["A1", "A2", "B1", "B2"]
        );
    }

    #[test]
    fn test_constants() {
        let game = init_stuff();

        assert_eq!(game.squares.len(), 81);
        assert_eq!(game.unitlist.len(), 27);
        for s in game.squares {
            dbg!(&game.unitlist);
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

    #[test]
    fn test_parse_grid_validation() {
        let state = init_stuff();
        assert_eq!(parse_grid("1234", &state).is_none(), true);
        assert_eq!(
            parse_grid(
                "123456789123456789123456789123456789123456789123456789123456789123456789123456789",
                &state
            )
            .is_some(),
            true
        );
        assert_eq!(parse_grid("1234567891234567891234567891234567891234567891234567891234567891234567891234567891", &state).is_none(), true);
    }

    // #[test]
    // fn test_parse_grid() {
    //     let state = init_stuff();
    //     assert_eq!(
    //         parse_grid(
    //             "123456789123456789123456789123456789123456789123456789123456789123456789123456789",
    //             &state
    //         )
    //         .is_some(),
    //         true
    //     );
    //     assert_eq!(parse_grid("1234567891234567891234567891234567891234567891234567891234567891234567891234567891", &state).is_none(), true);
    // }

}
