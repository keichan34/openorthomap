#!/bin/bash -e

INPUT="$1"
INPUT_B=$(basename -s ".tif" "$INPUT")
OUT_DEFAULT="$(dirname "$INPUT")/$INPUT_B.pmtiles"
OUT="${2:-$OUT_DEFAULT}"

if [[ -f "$OUT" ]]; then
  echo "Output file $OUT already exists, skipping"
  exit 0
fi

TMP=$(mktemp -d)

MBTILES="$TMP/$INPUT_B.mbtiles"
# create initial mbtiles with highest quality tiles only
gdal_translate -co "BLOCKSIZE=512" -co "TILE_FORMAT=WEBP" -co "QUALITY=70" -of mbtiles "$INPUT" "$MBTILES"

# create scaled-down tiles
gdaladdo -r nearest "$MBTILES"

PMTILES="$TMP/$INPUT_B.pmtiles"
# create pmtiles archive
pmtiles convert "$MBTILES" "$PMTILES"

cp "$PMTILES" "$OUT"

rm -r "$TMP"
