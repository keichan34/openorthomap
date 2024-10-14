#!/bin/bash -e

ID=$(npx ulid)

INPUT="$1"
input_filename=$(basename -s ".tif" "$INPUT")

echo "Processing $input_filename as $ID"

OUT_BASE=./api/v1
OUT_INDEX=$OUT_BASE/index
OUT_TILES=$OUT_BASE/tiles
TMP=$(mktemp -d)

# this creates a GeoJSON file with the convex hull of the input data
./scripts/create-extents.sh "$INPUT" "$OUT_TILES/$ID.geojson"
# create the preview PMTiles archive for that data
./scripts/create-pmtiles.sh "$INPUT" "$OUT_TILES/$ID.pmtiles"
# update the index with the new data
npx tsx ./scripts/update-index.mts "$OUT_TILES/$ID.geojson" "$OUT_TILES/$ID.pmtiles" "$OUT_INDEX"
# npx tsx ./scripts/create-h3-index.mts "$OUT_TILES/$ID.geojson" "$OUT_INDEX"

rm -r "$TMP"
