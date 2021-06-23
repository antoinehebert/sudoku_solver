use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::fs;
use std::time::Instant;

// TODO:
// - Make state singleton or global? Doesn't make sense to pass this down all the time.
// - Make search multi-thread?
// - We clone everything for simplicity, could we share elements to be more
//   efficient?
// - Use str instead of String when we can.
// - Use concat! instead of home brewed concat.

fn main() {
    let filename: String;

    match parse_args() {
        None => {
            println!("Expected a filename with a problem to solve.");
            return;
        }
        Some(fname) => filename = fname,
    };

    let problem = fs::read_to_string(filename).expect("Something went wrong reading the file.");

    let start = Instant::now();
    let state = init_stuff();
    let grid = solve(&problem.trim(), &state);
    println!("Elapsed time: {:?}.", start.elapsed());

    match grid {
        Some(g) => {
            println!("Solution:");
            display_grid(&g, &state);
        }
        None => println!("no solution :("),
    }
}

fn parse_args() -> Option<String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Expected a filename with a problem to solve.");
        return None;
    }

    let filename = &args[1];
    return Some(filename.to_string());
}

// cross("AB", "12") -> ["A1", "A2", "B1", "B2"]
fn cross(a: &Vec<String>, b: &Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for i in a {
        for j in b {
            result.push(concat(i, j));
        }
    }

    result
}

type Cell = String; // Row-Column id, e.g.: A1.
type CellValues = Vec<String>; // Possible values for a cell.

type Group = Vec<Cell>; // A row, column or 3x3 sub-grid.
type Groups = Vec<Group>;

type CellGroups = HashMap<Cell, Groups>;
type Peers = HashMap<Cell, Vec<Cell>>;
type Grid = HashMap<Cell, CellValues>;

#[derive(Debug)]
struct State {
    digits: Vec<String>,
    cols: Vec<String>,
    rows: Vec<String>,
    squares: Vec<Cell>,

    // .units["C2"] =
    // [
    //     ["A2", "B2", "C2", "D2", "E2", "F2", "G2", "H2", "I2"],
    //     ["C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9"],
    //     ["A1", "A2", "A3", "B1", "B2", "B3", "C1", "C2", "C3"]
    // ]
    // **NOTE:** C2 included.
    cell_groups: CellGroups,

    // .peers["C2"] =
    // [
    //     "A1", "A2", "A3", "B1", "B2", "B3", "C1", "C3", "C4", "C5", "C6", "C7", "C8", "C9",
    //     "D2", "E2", "F2", "G2", "H2", "I2"
    // ]
    // **NOTE:** C2 excluded.
    peers: Peers,
}

fn concat(s1: &String, s2: &String) -> String {
    format!("{}{}", s1, s2)
}

fn str_to_vec(s: &str) -> Vec<String> {
    s.chars().map(|s| s.to_string()).collect()
}

fn init_stuff() -> State {
    let digits: Vec<String> = str_to_vec("123456789");
    let cols: Vec<String> = str_to_vec("123456789");
    let rows: Vec<String> = str_to_vec("ABCDEFGHI");

    let squares = cross(&rows, &cols);

    // all groups
    let mut groups: Groups = vec![];
    for c in &cols {
        groups.push(cross(&rows, &vec![c.clone()]));
    }
    for r in &rows {
        groups.push(cross(&vec![r.clone()], &cols));
    }
    for rs in vec![str_to_vec("ABC"), str_to_vec("DEF"), str_to_vec("GHI")] {
        for cs in vec![str_to_vec("123"), str_to_vec("456"), str_to_vec("789")] {
            groups.push(cross(&rs, &cs));
        }
    }

    // cell groups
    let mut cell_groups: CellGroups = HashMap::new();

    for s in &squares {
        for u in &groups {
            if u.contains(s) {
                cell_groups
                    .entry(s.clone())
                    .or_insert_with(Vec::new)
                    .push(u.clone());
            }
        }
    }

    // peers
    let mut peers = Peers::new();
    for s in &squares {
        // flatten
        let mut new_units: Vec<Cell> = cell_groups
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
        digits: digits,
        cols: cols,
        rows: rows,
        squares: squares,
        peers: peers,
        cell_groups: cell_groups,
    }
}

fn parse_grid(grid_str: &str, state: &State) -> Option<Grid> {
    println!("Parsing {}\n", grid_str);
    if grid_str.len() != 81 {
        return None;
    }

    // Start with all possible values
    let mut result = Grid::new();

    // Fill grid with all digits first.
    for square in &state.squares {
        result.insert(square.clone(), state.digits.clone());
    }

    for (index, square) in state.squares.iter().enumerate() {
        let digit = &grid_str.chars().nth(index).unwrap().to_string();

        if state.digits.contains(digit) {
            if !assign(&mut result, square, digit, state) {
                return None;
            }
        }
    }

    Some(result)
}

/// Eliminate all the other `digit` from `grid[square]` and propagate.
fn assign(grid: &mut Grid, cell: &Cell, digit: &String, state: &State) -> bool {
    let mut other_digits = grid[cell].clone();
    other_digits.retain(|d| d != digit);

    if other_digits.iter().all(|d| eliminate(grid, cell, d, state)) {
        true
    } else {
        false
    }
}

/// Eliminate `digit` from `grid[square]` and propagate.
fn eliminate(grid: &mut Grid, cell: &Cell, digit: &String, state: &State) -> bool {
    // Eliminate!
    if !grid[cell].contains(digit) {
        return true; // Already eliminated.
    }

    grid.get_mut(cell).unwrap().retain(|d| d != digit);

    if grid[cell].len() == 0 {
        return false;
    }

    if grid[cell].len() == 1 {
        // Found a match, eliminate from peers.
        let d = &grid[cell][0].clone();
        // assert!(state.peers[cell].len() > 0); // remove!
        if !state.peers[cell]
            .iter()
            .all(|s2| eliminate(grid, s2, d, state))
        {
            return false;
        }
    }

    // If a unit is reduced to only one place for a value, then put it there.
    for unit in &state.cell_groups[cell] {
        let mut dplaces = Vec::new();
        for s in unit {
            if grid[s].contains(digit) {
                dplaces.push(s);
            }
        }

        if dplaces.len() == 0 {
            return false;
        }

        if dplaces.len() == 1 {
            // Digit can only be in one place in unit, assign it here.
            if !assign(grid, dplaces[0], digit, state) {
                return false;
            }
        }
    }

    true
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

fn display_grid(grid: &Grid, state: &State) {
    println!("{}", format_grid(grid, state));
}

fn format_grid(grid: &Grid, state: &State) -> String {
    let mut result = String::new();

    // 9 cols + 2 spaces on each side.
    let mut width = state.squares.iter().map(|s| grid[s].len()).max().unwrap();
    width += 2; // Padding

    // number header
    let mut digit_header: String = "".to_string();
    for (d_index, d) in state.cols.iter().enumerate() {
        if is_grid_boundary(d_index) {
            digit_header.push(' ');
        }
        digit_header.push_str(&center_string(&d, width));
    }
    writeln!(&mut result, "  {}", digit_header.trim_end()).unwrap();

    let mut line = vec!["-".repeat(3 * width); 3].join("+");
    line = format!("  +{}+", line);

    writeln!(&mut result, "{}", line).unwrap();
    for (row_index, row) in state.rows.iter().enumerate() {
        write!(&mut result, "{} ", row).unwrap();
        for (col_index, col) in state.cols.iter().enumerate() {
            if is_grid_boundary(col_index) {
                write!(&mut result, "|").unwrap();
            }

            // Would ideally use `{^<number>}` formatting instead of
            // `center_string`, but I don't think you can set the number
            // dynamically...
            write!(
                &mut result,
                "{}",
                center_string(&grid[&concat(&row, &col)].join(""), width)
            )
            .unwrap();
        }

        write!(&mut result, "|\n").unwrap();

        if (row_index + 1) % 3 == 0 {
            writeln!(&mut result, "{}", line).unwrap();
        }
    }

    result
}

fn solve(grid: &str, state: &State) -> Option<Grid> {
    match parse_grid(grid, state) {
        Some(new_grid) => search(&new_grid, state),
        None => {
            println!("Invalid Grid.");
            None
        }
    }
}

// Using depth-first search and propagation, try all possible values.
fn search(grid: &Grid, state: &State) -> Option<Grid> {
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

    for digit in digits.iter() {
        let mut new_grid = grid.clone();
        if assign(&mut new_grid, square, digit, state) {
            if let Some(result) = search(&new_grid, state) {
                return Some(result);
            }
        }
    }

    None
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
        for s in &game.squares {
            assert_eq!(game.cell_groups[s].len(), 3);
            assert_eq!(game.peers[s].len(), 20);
        }
        assert_eq!(
            game.cell_groups["C2"],
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
        );

        assert_eq!(
            &game.squares,
            &vec![
                "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9", "B1", "B2", "B3", "B4", "B5",
                "B6", "B7", "B8", "B9", "C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9", "D1",
                "D2", "D3", "D4", "D5", "D6", "D7", "D8", "D9", "E1", "E2", "E3", "E4", "E5", "E6",
                "E7", "E8", "E9", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "G1", "G2",
                "G3", "G4", "G5", "G6", "G7", "G8", "G9", "H1", "H2", "H3", "H4", "H5", "H6", "H7",
                "H8", "H9", "I1", "I2", "I3", "I4", "I5", "I6", "I7", "I8", "I9"
            ]
        );
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

    #[test]
    fn test_puzzle() {
        let state = init_stuff();
        let result = solve(
            "003020600900305001001806400008102900700000008006708200002609500800203009005010300",
            &state,
        );
        assert!(result.is_some());

        let expected_output = String::from(
            r#"    1  2  3   4  5  6   7  8  9
  +---------+---------+---------+
A | 4  8  3 | 9  2  1 | 6  5  7 |
B | 9  6  7 | 3  4  5 | 8  2  1 |
C | 2  5  1 | 8  7  6 | 4  9  3 |
  +---------+---------+---------+
D | 5  4  8 | 1  3  2 | 9  7  6 |
E | 7  2  9 | 5  6  4 | 1  3  8 |
F | 1  3  6 | 7  9  8 | 2  4  5 |
  +---------+---------+---------+
G | 3  7  2 | 6  8  9 | 5  1  4 |
H | 8  1  4 | 2  5  3 | 7  6  9 |
I | 6  9  5 | 4  1  7 | 3  8  2 |
  +---------+---------+---------+
"#,
        );

        assert_eq!(format_grid(&result.unwrap(), &state), expected_output);
    }
}
