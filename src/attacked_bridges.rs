use crate::coords::Coords;
use crate::hex_cells::{HexCells, Index};
use crate::neighbors::get_neighbors;

enum FindAttackedBridgesState {
    Found0,
    Found1,
    Found2,
}

pub fn find_attacked_bridges(cells: &HexCells, coords: Coords) -> Vec<Coords> {
    let center_index = cells.index_from_coords(coords);
    let search_color = match cells.get_color_at_index(center_index) {
        None => return vec![],
        Some(color) => color.opponent_color(),
    };

    // Essentially, the following code will search for the pattern [Some(search_color), None, Some(search_color)]
    // in the colors of the neighbors of `coords`. To find bridges that span the

    // Bridges may cross the end of the get_neighbors-iterator (e.g. the last element + the first two elements).
    // For this reason we extend the array of neighbors by the first two neighbors.
    // Unfortunately, we cannot know the number of neighbors (typically 6, but less on the edges).
    // Note that we initialize the array with `center_index` which is guaranteed to not match any part of the pattern.
    let mut neighbors: [Index; 8] = [center_index; 8];
    let neighbor_count = copy_into_array(&mut neighbors, 0, get_neighbors(cells, center_index));
    neighbors[neighbor_count] = neighbors[0];
    neighbors[neighbor_count + 1] = neighbors[1];

    println!("Center index {:?}", center_index);
    println!("Neighbors: {:?}", neighbors);

    let mut result = vec![];
    // `state` describes how many fields of the pattern [Some(search_color), None, Some(search_color)] we've already found.
    let mut state = FindAttackedBridgesState::Found0;

    for (i, neighbor) in neighbors.iter().enumerate() {
        let color = cells.get_color_at_index(*neighbor);
        println!("Iter {} {} {:?}", i, neighbor, color);
        match state {
            FindAttackedBridgesState::Found0 => {
                if color == Some(search_color) {
                    state = FindAttackedBridgesState::Found1;
                }
            }
            FindAttackedBridgesState::Found1 => {
                if color.is_none() {
                    state = FindAttackedBridgesState::Found2;
                } else if color != Some(search_color) {
                    state = FindAttackedBridgesState::Found0;
                }
                // else: remain in state Found1, because we already have the beginning of a possible new match.
            }
            FindAttackedBridgesState::Found2 => {
                if color == Some(search_color) {
                    result.push(cells.coords_from_index(neighbors[i - 1]));
                    // It is important to also find overlapping matches.
                    // The third part of the current bridge may be the first part of a new bridge.
                    state = FindAttackedBridgesState::Found1;
                } else {
                    state = FindAttackedBridgesState::Found0;
                }
            }
        }
    }

    result
}

fn copy_into_array<T, TIter: Iterator<Item = T>, const N: usize>(
    array: &mut [T; N],
    start_index: usize,
    iterator: TIter,
) -> usize {
    let mut index = start_index;
    for item in iterator {
        array[index] = item;
        index += 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use crate::{edges::set_edge_colors, Color};

    use super::*;
    const CENTER: Coords = Coords { row: 2, column: 2 };

    fn empty_cells_with_colored_edges() -> HexCells {
        let mut cells = HexCells::new(5);
        set_edge_colors(&mut cells);
        cells
    }

    #[test]
    fn test_coords_without_stone() {
        let cells = empty_cells_with_colored_edges();
        assert_eq!(find_attacked_bridges(&cells, CENTER), vec![]);
    }

    #[test]
    fn test_coords_without_neighbors() {
        let mut cells = empty_cells_with_colored_edges();
        let _ = cells.set_color_at_coords(CENTER, Color::White);
        assert_eq!(find_attacked_bridges(&cells, CENTER), vec![]);
    }

    #[test]
    fn test_simple_bridge_in_center() {
        //  a  b  c  d  e
        // 1\.  .  .  .  .\1
        //  2\.  .  .  ●  .\2
        //   3\.  .  ○  .  .\3
        //    4\.  .  ●  .  .\4
        let mut cells = empty_cells_with_colored_edges();
        let _ = cells.set_color_at_coords(Coords { row: 1, column: 3 }, Color::Black);
        let _ = cells.set_color_at_coords(Coords { row: 3, column: 2 }, Color::Black);
        let _ = cells.set_color_at_coords(CENTER, Color::White);

        assert_eq!(
            find_attacked_bridges(&cells, CENTER),
            vec![Coords { row: 2, column: 3 }]
        );
    }

    #[test]
    fn test_bridge_with_preceding_stone_in_center() {
        //  a  b  c  d  e
        // 1\.  .  .  .  .\1
        //  2\.  .  ●  .  .\2
        //   3\.  ●  ○  ●  .\3
        let mut cells = empty_cells_with_colored_edges();
        let _ = cells.set_color_at_coords(Coords { row: 2, column: 1 }, Color::Black);
        let _ = cells.set_color_at_coords(Coords { row: 1, column: 2 }, Color::Black);
        let _ = cells.set_color_at_coords(Coords { row: 2, column: 3 }, Color::Black);
        let _ = cells.set_color_at_coords(CENTER, Color::White);

        assert_eq!(
            find_attacked_bridges(&cells, CENTER),
            vec![Coords { row: 1, column: 3 }]
        );
    }

    #[test]
    fn test_non_bridge_with_preceding_stone_in_center() {
        //  a  b  c  d  e
        // 1\.  .  .  .  .\1
        //  2\.  .  ○  .  .\2
        //   3\.  ●  ○  ●  .\3
        let mut cells = empty_cells_with_colored_edges();
        let _ = cells.set_color_at_coords(Coords { row: 2, column: 1 }, Color::Black);
        let _ = cells.set_color_at_coords(Coords { row: 1, column: 2 }, Color::White);
        let _ = cells.set_color_at_coords(Coords { row: 2, column: 3 }, Color::Black);
        let _ = cells.set_color_at_coords(CENTER, Color::White);

        assert_eq!(find_attacked_bridges(&cells, CENTER), vec![]);
    }

    #[test]
    fn test_three_bridges_in_center() {
        //  a  b  c  d  e
        // 1\.  .  .  .  .\1
        //  2\.  .  .  ●  .\2
        //   3\.  ●  ○  .  .\3
        //    4\.  .  ●  .  .\4
        let mut cells = empty_cells_with_colored_edges();
        let _ = cells.set_color_at_coords(Coords { row: 2, column: 1 }, Color::Black);
        let _ = cells.set_color_at_coords(Coords { row: 1, column: 3 }, Color::Black);
        let _ = cells.set_color_at_coords(Coords { row: 3, column: 2 }, Color::Black);
        let _ = cells.set_color_at_coords(CENTER, Color::White);

        assert_eq!(
            find_attacked_bridges(&cells, CENTER),
            vec![
                Coords { row: 1, column: 2 },
                Coords { row: 2, column: 3 },
                Coords { row: 3, column: 1 },
            ]
        );
    }

    #[test]
    fn test_bridge_overlapping_from_last_to_first_neighbor() {
        //  a  b  c  d  e
        // 1\.  .  .  .  .\1
        //  2\.  .  ●  .  .\2
        //   3\.  .  ○  .  .\3
        //    4\.  ●  .  .  .\4
        let mut cells = empty_cells_with_colored_edges();
        let _ = cells.set_color_at_coords(Coords { row: 3, column: 1 }, Color::Black);
        let _ = cells.set_color_at_coords(Coords { row: 1, column: 2 }, Color::Black);
        let _ = cells.set_color_at_coords(CENTER, Color::White);

        assert_eq!(
            find_attacked_bridges(&cells, CENTER),
            vec![Coords { row: 2, column: 1 },]
        );
    }

    #[test]
    fn test_bridge_in_obtuse_corner() {
        //  a  b  c  d  e
        // 1\.  ○  .  .  .\1
        //  2\●  .  .  .  .\2
        let mut cells = empty_cells_with_colored_edges();
        let attacked_coords = Coords { row: 0, column: 1 };
        let _ = cells.set_color_at_coords(Coords { row: 1, column: 0 }, Color::Black);
        let _ = cells.set_color_at_coords(attacked_coords, Color::White);

        assert_eq!(
            find_attacked_bridges(&cells, attacked_coords),
            vec![Coords { row: 0, column: 0 },]
        );
    }

    #[test]
    fn test_bridge_next_to_obtuse_corner() {
        //  a  b  c  d  e
        // 1\○  .  .  .  .\1
        //  2\●  .  .  .  .\2
        let mut cells = empty_cells_with_colored_edges();
        let attacked_coords = Coords { row: 0, column: 0 };
        let _ = cells.set_color_at_coords(Coords { row: 1, column: 0 }, Color::Black);
        let _ = cells.set_color_at_coords(attacked_coords, Color::White);

        assert_eq!(
            find_attacked_bridges(&cells, attacked_coords),
            vec![Coords { row: 0, column: 1 },]
        );
    }

    #[test]
    fn test_bridge_to_own_edge() {
        //  a  b  c  d  e
        // 1\.  .  ○  .  .\1
        //  2\.  .  ●  .  .\2
        let mut cells = empty_cells_with_colored_edges();
        let attacked_coords = Coords { row: 0, column: 2 };
        let _ = cells.set_color_at_coords(Coords { row: 1, column: 2 }, Color::Black);
        let _ = cells.set_color_at_coords(attacked_coords, Color::White);

        assert_eq!(
            find_attacked_bridges(&cells, attacked_coords),
            vec![Coords { row: 0, column: 3 },]
        );
    }

    #[test]
    fn test_no_bridge_on_other_players_edge() {
        //  a  b  c  d  e
        // 1\.  .  .  .  .\1
        //  2\.  .  .  .  .\2
        //   3\○  ●  .  .  .\3
        let mut cells = empty_cells_with_colored_edges();
        let attacked_coords = Coords { row: 2, column: 0 };
        let _ = cells.set_color_at_coords(Coords { row: 2, column: 1 }, Color::Black);
        let _ = cells.set_color_at_coords(attacked_coords, Color::White);

        assert_eq!(find_attacked_bridges(&cells, attacked_coords), vec![]);
    }
}
