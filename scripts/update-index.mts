import fs from 'node:fs';
import path from 'node:path';
import type GeoJSON from 'geojson';
import { PMTiles, RangeResponse, Source } from 'pmtiles';
import { createHash } from 'node:crypto';

import vtpbf from 'vt-pbf';
import geojsonVt from 'geojson-vt';
import * as MVT from '@mapbox/vector-tile';

const IDX_MIN_ZOOM = 10;
const IDX_MAX_ZOOM = 15;

function md5To64BitInt(input: string): number {
  const hash = createHash('md5').update(input).digest('hex');
  const truncatedHash = parseInt(hash.slice(0, 13), 16);
  return truncatedHash;
}

class NodeJSFileSource implements Source {
  private readonly path: string;
  private readonly f: Promise<fs.promises.FileHandle>;

  constructor(path: string) {
    this.path = path;
    this.f = fs.promises.open(path, 'r');
  }

  getKey() {
    return this.path;
  }

  async getBytes(offset: number, length: number) {
    const f = await this.f;
    const buffer = Buffer.alloc(length);
    await f.read(buffer, 0, length, offset);
    return {
      data: buffer.buffer,
    };
  };
}

const VectorTile = MVT.VectorTile;

(async () => {
  const inputFile = process.argv[2];
  const inputPmtiles = process.argv[3];
  const indexDir = process.argv[4];

  const inputFileId = path.basename(inputFile, path.extname(inputFile));
  const featureBaseId = md5To64BitInt(inputFileId);

  const pmtiles = new PMTiles(new NodeJSFileSource(inputPmtiles));
  const pmtilesHeader = await pmtiles.getHeader();
  // console.log('pmtilesHeader:', pmtilesHeader);

  const data = JSON.parse(fs.readFileSync(inputFile, 'utf8')) as GeoJSON.FeatureCollection;
  for (let i = 0; i < data.features.length; i++) {
    const feature = data.features[i];
    feature.id = featureBaseId + i;

    feature.properties ??= {};
    feature.properties.tileset = inputFileId;
    feature.properties.minZoom = pmtilesHeader.minZoom;
    feature.properties.maxZoom = pmtilesHeader.maxZoom;
  }

  const indexMaxZoom = Math.min(IDX_MAX_ZOOM, pmtilesHeader.maxZoom);
  const tiles = geojsonVt(data, {
    // setting these two attributes will trigger all tiles to be generated
    indexMaxZoom,
    indexMaxPoints: 0,

    maxZoom: indexMaxZoom,
    extent: 2048,
    buffer: 64,
  });
  for (const {z, x, y} of tiles.tileCoords) {
    if (z < IDX_MIN_ZOOM) {
      console.warn(`Skipping ${z}/${x}/${y}`);
      continue;
    }
    const tile = tiles.getTile(z, x, y);
    if (!tile) {
      console.warn(`No tile for ${z}/${x}/${y}`);
      continue;
    }

    const outTilePath = path.join(indexDir, `${z}`, `${x}`, `${y}.pbf`);
    const outTileDir = path.dirname(outTilePath);
    await fs.promises.mkdir(outTileDir, { recursive: true });

    // TODO: read existing tile and merge if necessary
    const outTile = vtpbf.fromGeojsonVt({ 'index': tile });
    await fs.promises.writeFile(outTilePath, outTile);
  }

  await fs.promises.writeFile(
    path.join(indexDir, 'tiles.json'),
    JSON.stringify({
      "tilejson": "3.0.0",
      "name": "OpenOrthoMap Tileset Index",
      "tiles": [
        "oom-files:///api/v1/index/{z}/{x}/{y}.pbf"
      ],
      "minzoom": IDX_MIN_ZOOM,
      "maxzoom": IDX_MAX_ZOOM,
      "vector_layers": [
        {
          "id": "index",
          "description": "Index of tilesets",
          "fields": {
            "tileset": "string - the ULID of the tileset",
            "minZoom": "number",
            "maxZoom": "number",
          },
        },
      ]
    }),
    {
      flag: 'w',
    }
  );
})().then(() => {
  process.exit(0);
}).catch((error) => {
  console.error(error);
  process.exit(1);
});
