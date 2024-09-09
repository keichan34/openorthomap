import fs from 'node:fs';
import path from 'node:path';
import h3 from 'h3-js';

import type GeoJSON from 'geojson';

const H3_RESOLUTION = 9;

type IndexFile = {
  id: string; // H3 index
  count: number; // number of tilesets in this cell
  tilesets: string[]; // list of tileset IDs
}

(async () => {
  const inputFile = process.argv[2];
  const indexDir = process.argv[3];

  const inputFileId = path.basename(inputFile, path.extname(inputFile));

  const allCells = new Set<string>();
  const addParents = (cell: string) => {
    const res = h3.getResolution(cell);
    if (res === 0) {
      return;
    }
    const parent = h3.cellToParent(cell, res - 1);
    allCells.add(parent);
    addParents(parent);
  };
  const data = JSON.parse(fs.readFileSync(inputFile, 'utf8')) as GeoJSON.FeatureCollection;
  for (const feature of data.features) {
    const geom = feature.geometry;
    if (geom.type !== 'Polygon') {
      throw new Error(`Unsupported geometry type: ${geom.type}`);
    }
    const cells = h3.polygonToCells(geom.coordinates, H3_RESOLUTION, true);
    for (const cell of cells) {
      allCells.add(cell);
      addParents(cell);
    }
  }

  fs.mkdirSync(indexDir, { recursive: true });
  for (const cell of allCells) {
    const indexPath = path.join(indexDir, `${cell}.json`);
    let index: IndexFile;
    try {
      const indexRaw = fs.readFileSync(indexPath, 'utf8');
      index = JSON.parse(indexRaw);
    } catch (e) {
      index = {
        id: cell,
        count: 0,
        tilesets: [],
      };
    }
    const tilesetSet = new Set(index.tilesets);
    tilesetSet.add(inputFileId);
    index.count = tilesetSet.size;
    index.tilesets = Array.from(tilesetSet);
    fs.writeFileSync(indexPath, JSON.stringify(index));
  }
})().then(() => {
  process.exit(0);
}).catch((error) => {
  console.error(error);
  process.exit(1);
});
