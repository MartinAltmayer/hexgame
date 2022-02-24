use crate::{hex_cells::HexCells, Color, Coords};

/// Represents the four edges of the board.
///
/// The top and bottom edge belong to player `Black`, the other two edges belong to player `White`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Left,
    Top,
    Right,
    Bottom,
}

pub fn get_edges_of_color(color: Color) -> [Edge; 2] {
    match color {
        Color::Black => [Edge::Top, Edge::Bottom],
        Color::White => [Edge::Left, Edge::Right],
    }
}

/// Represents either a cell of the board or an edge.
/// This type is used in a few places that may return both coords and edges (e.g. get_neighbors).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordsOrEdge {
    Coords(Coords),
    Edge(Edge),
}

impl From<Edge> for CoordsOrEdge {
    fn from(edge: Edge) -> CoordsOrEdge {
        CoordsOrEdge::Edge(edge)
    }
}

impl From<Coords> for CoordsOrEdge {
    fn from(coords: Coords) -> CoordsOrEdge {
        CoordsOrEdge::Coords(coords)
    }
}

pub fn set_edge_colors(cells: &mut HexCells) {
    for color in [Color::Black, Color::White] {
        for edge in get_edges_of_color(color) {
            let index = cells.index_from_edge(edge);
            cells.set_color_at_index(index, color);
        }
    }
}
