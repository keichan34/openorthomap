import { FeatureCollection, LineString, Feature, MultiLineString, Position, Polygon } from "geojson";
import { greatCircle } from "@turf/great-circle";
import { flatten } from "@turf/flatten";
import { geojson, s1, s2 } from "s2js";

// We adjust the lines from great-circle to have lng > 180 and < -180 so that
// the polygons render correctly.
const overflowAntimeridianCrossings = (
  arcLines: FeatureCollection<LineString>,
) => {
  if (arcLines.features.length <= 4) return;

  const antimeridianCrossings: boolean[] = [];
  let antiCrossed = false;

  // find crossings
  let last = arcLines.features.at(-1)!.geometry.coordinates.at(-1)!;
  arcLines.features.forEach((f) => {
    const first = f.geometry.coordinates.at(0)!;
    if (
      (last[0] === 180 && first[0] === -180) ||
      (last[0] === -180 && first[0] === 180)
    ) {
      antiCrossed = !antiCrossed;
    }

    antimeridianCrossings.push(antiCrossed);
    last = f.geometry.coordinates.at(-1)!;
  });

  // wrap lats
  arcLines.features.forEach((f, fi) => {
    if (!antimeridianCrossings[fi]) return;
    f.geometry.coordinates.forEach((v) => (v[0] += 360));
  });
};

// The 'top' and 'bottom' faces are special-cased since all all points have equal lat.
// we add in an additional line segment so the top and bottom of the planar map are covered
// by the polygon.
const fixPolarFaces = (
  cellid: bigint,
  arcs: Feature<LineString | MultiLineString>[],
) => {
  const POLAR_FACES = [s2.cellid.fromFace(2), s2.cellid.fromFace(5)];
  if (!POLAR_FACES.includes(cellid)) return;

  arcs.forEach((arc: Feature<LineString | MultiLineString>) => {
    if (arc.geometry.type === "MultiLineString") {
      const A = arc.geometry.coordinates[0].at(-1)!;
      const B = arc.geometry.coordinates[1].at(0)!;

      // sanity checks
      if (!A || Math.abs(A[0]) !== 180 || !B || Math.abs(B[0]) !== 180) {
        return;
      }

      // the target polar latitude
      const E = Math.sign(A[0]) * 90;

      // draw a line to the pole, across, and back again
      arc.geometry.coordinates = [
        arc.geometry.coordinates[0],
        [A, [A[0], E], [B[0], E], B],
        arc.geometry.coordinates[1],
      ];
    }
  });
};

// When drawing to a pole the lng is not relevant, we borrow it from the previous point
// to avoid line drawing issues.
const fixPoles = (points: Position[]) => {
  points.forEach((p, i) => {
    if (Math.abs(p[1]) === 90) p[0] = points.at(i - 1)![0];
  });
};

// The great-circle lib can sometimes use -180 lng instead of +180 and vice-versa
const fixFalseAntimeridianCrossings = (points: Position[]) => {
  const exterior = points.filter((p) => Math.abs(p[0]) === 180);
  if (exterior.length !== 2) return;
  if (Math.sign(exterior[0][0]) !== Math.sign(exterior[1][0])) return;

  const interior = points.filter((p) => Math.abs(p[0]) !== 180);
  if (interior.length !== 2) return;
  if (Math.sign(interior[0][0]) !== Math.sign(interior[1][0])) return;

  exterior.forEach((p) => (p[0] = Math.sign(interior[0][0]) * 180));
};

// The great-circle lib can generate a linestring with two identical points, remove them.
const removeRedundantArcs = (arcs: Feature<LineString | MultiLineString>[]) => {
  arcs.forEach((arc: Feature<LineString | MultiLineString>) => {
    if (arc.geometry.type !== "MultiLineString") return;
    if (arc.geometry.coordinates.length !== 2) return;
    arc.geometry.coordinates = arc.geometry.coordinates.filter((ls) => {
      if (ls.length !== 2) return true;
      if (ls[0][0] === ls[1][0] && ls[0][1] === ls[1][1]) return false;
      return true;
    });
  });
};


export const getCellVisualization = (union: s2.CellUnion): FeatureCollection => {
  const features = [...union].map((cellid): Feature<Polygon> => {
    const cell = s2.Cell.fromCellID(cellid);
    const poly = geojson.toGeoJSON(cell) as Polygon;
    const ring = poly.coordinates[0];

    fixPoles([ring[0], ring[1], ring[2], ring[3]]);
    fixFalseAntimeridianCrossings([ring[0], ring[1], ring[2], ring[3]]);

    const level = cell.level;
    const npoints = 20 + (30 - level) * 3; // more interpolated points for larger cells
    const arc0 = greatCircle(ring[0], ring[1], { npoints });
    const arc1 = greatCircle(ring[1], ring[2], { npoints });
    const arc2 = greatCircle(ring[2], ring[3], { npoints });
    const arc3 = greatCircle(ring[3], ring[0], { npoints });

    removeRedundantArcs([arc0, arc1, arc2, arc3]);
    fixPolarFaces(cellid, [arc0, arc1, arc2, arc3]);

    const arcLines: FeatureCollection<LineString> = {
      type: "FeatureCollection",
      features: [
        ...flatten(arc0).features,
        ...flatten(arc1).features,
        ...flatten(arc2).features,
        ...flatten(arc3).features,
      ],
    };

    overflowAntimeridianCrossings(arcLines);

    const coordinates = arcLines.features
      .map((f) => f.geometry.coordinates)
      .flat(1);

    const center = s2.LatLng.fromPoint(cell.center());

    return {
      type: "Feature",
      id: cell.id.toString(),
      geometry: {
        type: "Polygon",
        coordinates: [coordinates],
      },
      properties: {
        level: cell.level,
        token: s2.cellid.toToken(cell.id),
        centerLng: s1.angle.degrees(center.lng),
        centerLat: s1.angle.degrees(center.lat),
      },
    };
  });
  return {
    type: "FeatureCollection",
    features: features,
  };
};
