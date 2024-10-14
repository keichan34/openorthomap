# openorthomap

Scripts to manage and update a catalog of raster tilesets.

## Basic Architecture

Each raster tileset is managed as a PMTiles archive. The extent of the tileset is created as a simple vector polygon, exported, and added to the index. The index consists of Mapbox Vector Tile (MVT) files, and the extent corresponding to the raster tileset is added to the MVT index.

## Updating the Index

```
$ scripts/process-geotiff.sh [inputGeoTIFF]
```

The index will be in `./api/v1/index/{z}/{x}/{y}.pbf`, and the PMTiles archives will be in `./api/v1/tiles/{tilesetId}.pmtiles`.
