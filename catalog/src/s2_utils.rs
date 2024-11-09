use std::{collections::HashSet, usize};

use geo::{polygon, BooleanOps, BoundingRect, GeodesicArea, MultiPolygon, Polygon};
use s2::{
    self,
    cell::{self, Cell},
    cellid::CellID,
    cellunion::CellUnion,
};

const BASE_CELL_LEVEL: u8 = 12;

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

fn get_children_at_base(cells: Vec<CellID>) -> HashSet<CellID> {
    let mut all_children = HashSet::new();
    // let cell: Cell = cell.into();
    for cell_id in cells {
        let cell = Cell::from(cell_id);
        if cell.level() == BASE_CELL_LEVEL {
            all_children.insert(cell_id);
            continue;
        }

        let children = cell.children().unwrap();
        let children_ids: Vec<CellID> = children.iter().map(|child| child.id).collect();
        all_children.extend(children_ids);
    }
    all_children
}

/**
 * Function to calculate the S2 cells that cover the particular
 * input polygon.
 */
pub fn covering_s2_cells(polygon: &MultiPolygon) -> Vec<CellID> {
    // because s2 works with bounding boxes, we first get the bounding box
    // of the input polygon
    let bounding_rect = polygon.bounding_rect().unwrap();

    let bbox_region_coverer = s2::region::RegionCoverer {
        min_level: 0,
        max_level: BASE_CELL_LEVEL,
        level_mod: 1,
        max_cells: usize::MAX,
    };
    let s2_rect = s2::rect::Rect::from_degrees(
        bounding_rect.min().y,
        bounding_rect.min().x,
        bounding_rect.max().y,
        bounding_rect.max().x,
    );
    let bbox_cells = bbox_region_coverer.covering(&s2_rect);
    // println!("Bounding box cells: {:?}", bbox_cells);
    let all_children = get_children_at_base(bbox_cells.0);
    // println!("All children: {:?}", all_children);
    let intersecting_cells: Vec<CellID> = all_children
        .into_iter()
        .filter(|cell| coverage_of_cell(polygon, cell) > 0.0_f64)
        .collect();
    // println!("Intersecting cells: {:?}", intersecting_cells);

    let mut intersection_cell_union = CellUnion(intersecting_cells);
    intersection_cell_union.normalize();
    // println!("Intersection cell union: {:?}", intersection_cell_union);
    let expanded_cells = expand_cell_parents(intersection_cell_union.0);

    let mut sorted_cells: Vec<_> = expanded_cells.into_iter().collect();
    sorted_cells.sort();
    sorted_cells
}

/**
 * Function to calculate the coverage of a polygon over a given S2 cell.
 */
pub fn coverage_of_cell(polygon: &MultiPolygon, cell_id: &CellID) -> f64 {
    let cell: s2::cell::Cell = cell_id.into();
    let cell_polygon = polygon![
        (x: cell.longitude(0, 0).deg(), y: cell.latitude(0, 0).deg()),
        (x: cell.longitude(0, 1).deg(), y: cell.latitude(0, 1).deg()),
        (x: cell.longitude(1, 1).deg(), y: cell.latitude(1, 1).deg()),
        (x: cell.longitude(1, 0).deg(), y: cell.latitude(1, 0).deg()),
        (x: cell.longitude(0, 0).deg(), y: cell.latitude(0, 0).deg()),
    ];
    let cell_polygon_area = cell_polygon.geodesic_area_signed().abs();
    let intersection = polygon.intersection(&cell_polygon);
    let area = intersection.geodesic_area_signed().abs();
    // println!(
    //     "cell: {:?} cell polygon area: {} \n polygon area: {} \n intersection area: {}",
    //     cell_id,
    //     cell_polygon_area,
    //     polygon.geodesic_area_signed().abs(),
    //     area,
    // );
    area / cell_polygon_area
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
            "3", "34", "35", "353", "353c", "353d", "353d204", "353d21", "353d21c", "353d24",
            "353d3", "353d4", "354",
        ];

        assert_eq!(expected_cells, cells);
    }
}
