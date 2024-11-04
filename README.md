# OpenOrthoMap

Tools for creating / maintaining an [OpenOrthoMap](https://www.openorthomap.org) instance.

## Basic Architecture

* [The Catalog](https://www.openorthomap.org/posts/2024-11-catalog/)
* The Data
  * Each raster tileset is managed as a PMTiles archive. The extent of the tileset is created as a simple vector polygon, exported, and added to the index. The index consists of Mapbox Vector Tile (MVT) files, and the extent corresponding to the raster tileset is added to the MVT index.

## Developing

TODO

## Adding a new tileset

```
$ updater [inputGeoTIFF]
```

These tools assume the API is located in `$PWD/api/v1/`: the catalog will be in `/api/v1/catalog/{cell id}.csv`, and files will be in `/api/v1/files/{file ID}.{ext}`
