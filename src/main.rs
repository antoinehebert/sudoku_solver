use std::collections::HashMap;
use std::env;
use std::fs;
use std::time::Instant;

// TODO:
//     - Make state singleton
//     - Make search multithread?
//     - We clone everything for simplicity, could we share elements to be more
//       efficient?

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Expected a filename with a problem to solve.");
        return;
    }

    let filename = &args[1];
    let problem = fs::read_to_string(filename).expect("Something went wrong reading the file.");

    let start = Instant::now();
    let state = init_stuff();
    let grid = solve(&problem.trim(), &state);
    println!("Elapsed time: {:?} secs.", start.elapsed());

    match grid {
        Some(g) => {
            println!("Solution:");
            display(&g, &state);
        }
        None => println!("no solution :("),
    }
}

// ("AB", "12") -> ["A1", "A2", "B1", "B2"]
fn cross(a: &Vec<String>, b: &Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for i in a {
        for j in b {
            result.push(concat(i, j));
        }
    }

    result
}

type Square = String; // Row-Column id, e.g.: A1.
type SquareResult = Vec<String>; // Possible values for a square.

type UnitsList = Vec<SquareResult>; // A row, column or 3x3 square.

type UnitlistsForSquare = HashMap<Square, UnitsList>;
type UnitsForSquare = HashMap<Square, SquareResult>;

#[derive(Debug)]
struct State {
    digits: Vec<String>,
    cols: Vec<String>,
    rows: Vec<String>,
    squares: Vec<String>,
    units: UnitlistsForSquare,
    peers: UnitsForSquare,
}

// I can't believe there's no better way of doing this!?
fn concat(s1: &String, s2: &String) -> String {
    let mut new_str = s1.clone();
    new_str.push_str(s2);
    return new_str;
}

fn str_to_vec(s: &str) -> Vec<String> {
    s.chars().map(|s| s.to_string()).collect()
}

fn init_stuff() -> State {
    let digits: Vec<String> = str_to_vec("123456789");
    let cols: Vec<String> = str_to_vec("123456789");
    let rows: Vec<String> = str_to_vec("ABCDEFGHI");

    let squares = cross(&rows, &cols);

    // unitlist
    let mut unitlist: UnitsList = vec![];
    for c in &cols {
        unitlist.push(cross(&rows, &vec![c.clone()]));
    }
    for r in &rows {
        unitlist.push(cross(&vec![r.clone()], &cols));
    }
    for rs in vec![str_to_vec("ABC"), str_to_vec("DEF"), str_to_vec("GHI")] {
        for cs in vec![str_to_vec("123"), str_to_vec("456"), str_to_vec("789")] {
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

    let mut peers = UnitsForSquare::new();
    for s in &squares {
        // flatten
        let mut new_units: SquareResult = units
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
        peers: peers,
        units: units,
    }
}

fn parse_grid(grid_str: &str, state: &State) -> Option<UnitsForSquare> {
    println!("Parsing {}\n", grid_str);
    if grid_str.len() != 81 {
        return None;
    }

    // Start with all possible values
    let mut result = UnitsForSquare::new();

    // Fill grid with all digits first.
    for square in &state.squares {
        result.insert(square.clone(), state.digits.clone());
    }

    for (index, square) in state.squares.iter().enumerate() {
        let digit = &grid_str.chars().nth(index).unwrap().to_string();

        if state.digits.contains(digit) {
            if let None = assign(&mut result, square, digit, state) {
                return None;
            }
        }
    }

    Some(result)
}

fn assign(
    grid: &mut UnitsForSquare,
    square: &Square,
    digit: &String,
    state: &State,
) -> Option<()> {
    let mut other_digits = grid[square].clone();
    other_digits.retain(|d| d != digit);
    for d in &other_digits {
        if let None = eliminate(grid, square, d, state) {
            return None;
        }
    }

    Some(())
}

fn eliminate(
    grid: &mut UnitsForSquare,
    square: &Square,
    digit: &String,
    state: &State,
) -> Option<()> {
    // Eliminate!
    if !grid[square].contains(digit) {
        return Some(()); // Already eliminated.
    }

    grid.get_mut(square).unwrap().retain(|d| d != digit);

    if grid[square].len() < 1 {
        return None;
    } else if grid[square].len() == 1 {
        // Found a match, eliminate from peers.
        let d = &grid[square][0].clone();
        for s2 in &state.peers[square] {
            if let None = eliminate(grid, s2, d, state) {
                return None;
            }
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
            return None;
        } else if dplaces.len() == 1 {
            // Digit can only be in one place in unit, assign it here.
            if let None = assign(grid, dplaces[0], digit, state) {
                return None;
            }
        }
    }

    Some(())
}

fn center_string(s: &String, number_of_chars: usize) -> String {
    if number_of_chars <= s.len() {
        return s.clone();
    }

    let spaces_right = (number_of_chars - s.len()) / 2;
    let spaces_left = spaces_right + (number_of_chars - s.len()) % 2;

    format!(
        "{}{}{}",
        " ".repeat(spaces_left),
        s.clone(),
        " ".repeat(spaces_right)
    )
}

fn is_grid_boundary(col_index: usize) -> bool {
    (col_index + 1) % 3 == 1
}

fn display(grid: &UnitsForSquare, state: &State) {
    // 9 digits + 2 spaces on each side.
    let mut width = state.squares.iter().map(|s| grid[s].len()).max().unwrap();
    width += 2; // Padding

    // number header
    let mut digit_header: String = "".to_string();
    for (d_index, d) in state.digits.iter().enumerate() {
        if is_grid_boundary(d_index) {
            digit_header.push(' ');
        }
        digit_header.push_str(&center_string(&d, width));
    }
    println!("  {}", digit_header);

    let mut line = vec!["-".repeat(3 * width); 3].join("+");
    line = format!("  +{}+", line);

    println!("{}", line);
    for (row_index, row) in state.rows.iter().enumerate() {
        print!("{} ", row);
        for (col_index, col) in state.cols.iter().enumerate() {
            if is_grid_boundary(col_index) {
                print!("|");
            }

            // Would ideally use `{^<number>}` formatting instead of
            // `center_string`, but I don't think you can set the number
            // dynamically...
            print!(
                "{}",
                center_string(&grid[&concat(&row, &col)].join(""), width)
            );
        }

        print!("|\n");

        if (row_index + 1) % 3 == 0 {
            println!("{}", line);
        }
    }
}

fn solve(grid: &str, state: &State) -> Option<UnitsForSquare> {
    match parse_grid(grid, state) {
        Some(new_grid) => search(&Some(new_grid), state),
        None => {
            println!("Invalid Grid.");
            None
        }
    }
}

// Using depth-first search and propagation, try all possible values.
fn search(grid: &Option<UnitsForSquare>, state: &State) -> Option<UnitsForSquare> {
    match grid {
        Some(grid) => {
            // Solved!
            if grid.values().all(|v| v.len() == 1) {
                return Some(grid.clone());
            }
            // Chose the unfilled square with the fewest possibilities.
            let (square, digits) = grid
            .iter()
            .filter(|(_k, v)| v.len() > 1)
            .min_by(|(_k1, v1), (_k2, v2)| v1.len().cmp(&v2.len()))
            .unwrap();
            for digit in digits.into_iter() {
                // display(&grid, &state);
                // println!("digging for {}: {}", square, digit);
                let mut new_grid = grid.clone();
                if let None = assign(&mut new_grid, square, digit, state) {
                    return None;
                }

                if let Some(result) = search(&Some(new_grid), state) {
                    return Some(result);
                }
            }
            None
        }
        None => None,
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
        assert_eq!(
            cross(&str_to_vec("AB"), &str_to_vec("12")),
            ["A1", "A2", "B1", "B2"]
        );
    }

    #[test]
    fn test_constants() {
        let game = init_stuff();

        assert_eq!(game.squares.len(), 81);
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

    #[test]
    fn test_parse_grid_validation() {
        let state = init_stuff();
        assert_eq!(parse_grid("1234", &state).is_none(), true);
        assert_eq!(
            parse_grid(
                "123456789123456789123456789123456789123456789123456789123456789123456789123456789",
                &state
            )
            .is_none(),
            true
        );
        let grid =
            "003020600900305001001806400008102900700000008006708200002609500800203009005010300";
        assert_eq!(parse_grid(grid, &state).is_some(), true);
    }

    #[test]
    fn test_center_string() {
        assert_eq!(center_string(&"asdf".to_string(), 2), "asdf");
        assert_eq!(center_string(&"asdf".to_string(), 4), "asdf");
        assert_eq!(center_string(&"asdf".to_string(), 5), " asdf");
        assert_eq!(center_string(&"asdf".to_string(), 6), " asdf ");
        assert_eq!(center_string(&"asdf".to_string(), 7), "  asdf ");
        assert_eq!(center_string(&"asdf".to_string(), 10), "   asdf   ");
        assert_eq!(center_string(&"asdf".to_string(), 11), "    asdf   ");
    }
}
