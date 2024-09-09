#!/bin/bash -e

INPUT="$1"
INPUT_B=$(basename -s ".tif" "$INPUT")
OUT_DEFAULT="$(dirname "$INPUT")/$INPUT_B.geojson"
OUT="${2:-$OUT_DEFAULT}"

if [[ -f "$OUT" ]]; then
  echo "Output file $OUT already exists, skipping"
  exit 0
fi

TMP=$(mktemp -d)

# reproject to EPSG:3857 (pseudo-mercator) so we get units=meters
PROJECTED="$TMP/$INPUT_B.proj3857.tif"
gdalwarp -t_srs EPSG:3857 "$INPUT" "$PROJECTED"

# make a mask of the input data
MASK="$TMP/$INPUT_B.mask.tif"
gdal_calc -A "$PROJECTED" --outfile="$MASK" --calc="A>0"

# resample so the output resolution is 1mx1m
RESAMPLE="$TMP/$INPUT_B.resample.tif"
gdalwarp -tr 1 1 "$MASK" "$RESAMPLE"

# Convert to GeoJSON
GEOJSON="$TMP/$INPUT_B.mask.geojson"
gdal_polygonize "$RESAMPLE" -f "GeoJSON" "$GEOJSON"

# Calculate the convex hull, simplify it, reproject to EPSG:4326, then output it
HULL="$TMP/$INPUT_B.hull.geojson"
ogr2ogr \
  -f "GeoJSON" \
  "$HULL" \
  "$GEOJSON" \
  -dialect sqlite \
  -t_srs EPSG:4326 \
  -sql "SELECT ST_Simplify(ST_ConvexHull(ST_Union(geometry)), 2) AS geometry FROM out WHERE DN = 1"

mkdir -p "$(dirname "$OUT")"
cp "$HULL" "$OUT"

rm -r "$TMP"
