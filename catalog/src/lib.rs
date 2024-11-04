use std::usize;

use geo::{polygon, BoundingRect, Intersects, Polygon, Rect};
use s2::{self, cell::Cell, cellid::CellID};

/**
 * Function to calculate the S2 cells that cover the particular
 * input polygon.
 */
pub fn covering_s2_cells(polygon: &Polygon) -> Vec<CellID> {
    // because s2 works with bounding boxes, we first get the bounding box
    // of the input polygon
    let bounding_rect = polygon.bounding_rect().unwrap();

    let region_coverer = s2::region::RegionCoverer {
        min_level: 15,
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

    // Now, we need to filter the cells based on whether they actually intersect
    // with the input polygon or not. We do this by checking if the cell's bounding
    // rectangle intersects with the input polygon's bounding rectangle.
    let mut intersecting_cells: Vec<CellID> = Vec::new();
    for cell_id in cells.0 {
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
            intersecting_cells.push(cell_id);
        }
    }
    // println!("filtered cells: {:?}", intersecting_cells.len());

    intersecting_cells
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
        let cells = covering_s2_cells(&polygon);
        assert_eq!(cells.len(), 20);
        // println!("{:?}", cells);
    }
}
