use crate::errors::Result;
use geo::{polygon, BooleanOps, Geometry, GeometryCollection, MultiPolygon};
use geojson::{quick_collection, GeoJson};

pub fn geojson_to_union(geojson: String) -> Result<MultiPolygon<f64>> {
    let geojson = geojson.parse::<GeoJson>()?;
    let collection: GeometryCollection<f64> = quick_collection(&geojson)?;
    let mut union_geom: Option<MultiPolygon<f64>> = None;

    for geom in collection.iter() {
        match (geom, union_geom.clone()) {
            (Geometry::Polygon(ref geom), None) => union_geom = Some(geom.clone().into()),
            (Geometry::Polygon(ref geom), Some(ref mut union)) => {
                let union = union.union(geom);
                union_geom = Some(union);
            }
            (Geometry::MultiPolygon(ref geom), None) => union_geom = Some(geom.clone()),
            (Geometry::MultiPolygon(ref geom), Some(ref mut union)) => {
                let union = union.union(geom);
                union_geom = Some(union);
            }
            _ => (),
        }
    }
    Ok(union_geom.unwrap_or(polygon![].into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geojson_to_union() {
        let geojson_str = r#"
        {
            "type": "FeatureCollection",
            "features": [
                {"type":"Feature","properties":{},"geometry":{"coordinates":[[[130.58251520928872,30.33844626300842],[130.58251520928872,30.328505838499822],[130.5956914915006,30.328505838499822],[130.5956914915006,30.33844626300842],[130.58251520928872,30.33844626300842]]],"type":"Polygon"}},
                {"type":"Feature","properties":{},"geometry":{"coordinates":[[[130.58817598034045,30.334908353854885],[130.58817598034045,30.325721486852416],[130.59921020236277,30.325721486852416],[130.59921020236277,30.334908353854885],[130.58817598034045,30.334908353854885]]],"type":"Polygon"}}
            ]
        }
        "#;
        let union = geojson_to_union(geojson_str.to_string()).unwrap();

        let merged_geojson_str = r#"
        {"coordinates":[[[[130.58251520928988,30.328505838518055],[130.58251520928988,30.338446262986423],[130.59569149148828,30.338446262986423],[130.59569149148828,30.334908353829547],[130.5992102023616,30.334908353829547],[130.5992102023616,30.325721486874414],[130.58817598036202,30.325721486874414],[130.58817598036202,30.328505838518055],[130.58251520928988,30.328505838518055]]]],"type":"MultiPolygon"}
        "#;
        let merged_geojson = merged_geojson_str.parse::<GeoJson>().unwrap();
        match merged_geojson {
            GeoJson::Geometry(merged) => {
                let merged: MultiPolygon<f64> = merged.try_into().unwrap();
                assert_eq!(union, merged);
            }
            _ => {
                panic!("Expected a MultiPolygon, got something else");
            }
        }
    }
}
