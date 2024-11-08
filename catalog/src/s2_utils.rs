use std::{collections::HashSet, usize};

use geo::{polygon, BoundingRect, Intersects, MultiPolygon};
use s2::{self, cell::Cell, cellid::CellID};

// fn expand_cell_children_to_level(input_cell: CellID, level: u64) -> HashSet<CellID> {
//     let mut expanded_cells: HashSet<CellID> = HashSet::new();
//     if input_cell.level() == level {
//         expanded_cells.insert(input_cell);
//     } else {
//         let children = input_cell.children();
//         for child in children {
//             expanded_cells.insert(child);
//             expanded_cells.extend(expand_cell_children_to_level(child, level));
//         }
//     }
//     expanded_cells
// }

fn expand_cell_parents(cells: Vec<CellID>) -> HashSet<CellID> {
    let mut expanded_cells: HashSet<CellID> = HashSet::new();
    for cell in cells {
        let current_level = cell.level();
        for level in (0..current_level).rev() {
            let parent = cell.parent(level);
            expanded_cells.insert(parent);
        }
    }
    expanded_cells
}

/**
 * Function to calculate the S2 cells that cover the particular
 * input polygon.
 */
pub fn covering_s2_cells(polygon: &MultiPolygon) -> Vec<CellID> {
    // because s2 works with bounding boxes, we first get the bounding box
    // of the input polygon
    let bounding_rect = polygon.bounding_rect().unwrap();

    let region_coverer = s2::region::RegionCoverer {
        min_level: 0,
        max_level: 15,
        level_mod: 1,
        max_cells: usize::MAX,
    };
    let s2_rect = s2::rect::Rect::from_degrees(
        bounding_rect.min().y,
        bounding_rect.min().x,
        bounding_rect.max().y,
        bounding_rect.max().x,
    );
    let cells = region_coverer.covering(&s2_rect);
    // println!("cells: {:?}", cells.0.len());

    // let mut expanded_cells: HashSet<CellID> = HashSet::new();
    // for cell_id in cells.0 {
    //     expanded_cells.extend(expand_cell_children_to_level(cell_id, 15));
    // }
    // expanded_cells = expand_cell_parents(cells.0);
    let expanded_cells = expand_cell_parents(cells.0);

    // Now, we need to filter the cells based on whether they actually intersect
    // with the input polygon or not. We do this by checking if the cell's bounding
    // rectangle intersects with the input polygon's bounding rectangle.
    let mut intersecting_cells: HashSet<CellID> = HashSet::new();
    for cell_id in expanded_cells {
        let cell = Cell::from(cell_id);
        let cell_polygon = polygon![
            (x: cell.longitude(0, 0).deg(), y: cell.latitude(0, 0).deg()),
            (x: cell.longitude(0, 1).deg(), y: cell.latitude(0, 1).deg()),
            (x: cell.longitude(1, 1).deg(), y: cell.latitude(1, 1).deg()),
            (x: cell.longitude(1, 0).deg(), y: cell.latitude(1, 0).deg()),
        ];
        // println!(
        //     "cell: {:?} cell_rect: {:?}",
        //     cell_id.to_token(),
        //     cell_polygon
        // );
        if cell_polygon.intersects(polygon) {
            intersecting_cells.insert(cell_id);
        }
    }
    // println!("filtered cells: {:?}", intersecting_cells.len());

    // now, we need to expand intersecting_cells to include all child cells up to level 15

    let mut cells: Vec<_> = intersecting_cells.into_iter().collect();
    cells.sort();
    cells
}

#[cfg(test)]
mod tests {
    use geo::polygon;

    use super::*;

    #[test]
    fn test_covering_s2_cells() {
        let polygon = polygon![
            (x: 130.59011515516988, y: 30.338085913448012),
            (x: 130.58451092777312, y: 30.33241331762953),
            (x: 130.58856924351488, y: 30.328005163515115),
            (x: 130.5965268876003, y: 30.332905838105475),
            (x: 130.59470078699604, y: 30.33793038867809),
            (x: 130.59011515516988, y: 30.338085913448012),
        ];
        let cells = covering_s2_cells(&polygon.into())
            .into_iter()
            .map(|cell| cell.to_token())
            .collect::<Vec<String>>();
        let expected_cells = vec![
            "3", "34", "35", "353", "353c", "353d", "353d203", "353d2034", "353d2035", "353d204",
            "353d204b", "353d204c", "353d205", "353d21", "353d21b", "353d21b3", "353d21b4",
            "353d21c", "353d21c9", "353d21cc", "353d21d", "353d24", "353d3", "353d4", "354",
        ];

        assert_eq!(expected_cells, cells);
    }
}
