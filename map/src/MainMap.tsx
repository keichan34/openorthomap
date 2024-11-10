import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import maplibregl, { GeoJSONSource } from "maplibre-gl";
import "maplibre-gl/dist/maplibre-gl.css";
import type { Geometry } from "geojson";
import { Protocol } from "pmtiles";
import { s2, geojson } from 's2js';
import { getCellVisualization } from "./lib/s2";

const pmtilesProtocol = new Protocol();
maplibregl.addProtocol("pmtiles", pmtilesProtocol.tile);

const S2_BASE_LEVEL = 12;

// maplibregl.addProtocol("oom-files", async (params, abort) => {
//   const prefix = import.meta.env.VITE_FILES_URL;
//   const inputUrl = new URL(params.url);
//   const pathname = inputUrl.pathname.replace(/^\/+/, "");
//   const url = `${prefix}/${pathname}`;
//   const response = await fetch(url, { signal: abort.signal });
//   if (!response.ok) {
//     throw new Error(`Failed to fetch ${url}: ${response.statusText}`);
//   }
//   if (params.type === "json") {
//     let text = await response.text();
//     text = text.replace(/"oom-files:\/\/\//g, `"${prefix}/`);
//     return {
//       data: JSON.parse(text),
//     };
//   } else if (params.type === "string") {
//     return {
//       data: await response.text(),
//     };
//   } else if (params.type === "arrayBuffer") {
//     return {
//       data: await response.arrayBuffer(),
//     };
//   } else {
//     throw new Error(`Unknown type: ${params.type}`);
//   }
// });

const MainMap: React.FC = () => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [loadedMap, setLoadedMap] = useState<maplibregl.Map | undefined>(undefined);
  const [tilesetHashes, setTilesetHashes] = useState<Set<string>>(new Set());

  useLayoutEffect(() => {
    if (!containerRef.current) {
      return;
    }

    const map = new maplibregl.Map({
      container: containerRef.current,
      style: "/style.json",
      hash: "map",
      center: [135, 35],
      zoom: 5,
      minZoom: 4,
    });
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (window as any)._mainMap = map;

    map.addControl(new maplibregl.NavigationControl(), "top-right");
    map.addControl(new maplibregl.GeolocateControl({
      positionOptions: {
        enableHighAccuracy: true,
      },
      trackUserLocation: true,
    }), "top-right");
    map.addControl(new maplibregl.ScaleControl(), "bottom-left");

    map.on("load", () => {
      console.log("Map loaded");
      setLoadedMap(map);
    });

    map.on("moveend", () => {
    });

    return () => {
      map.remove();
    }
  }, []);

  const refreshVisibleTilesets = useCallback(async (map: maplibregl.Map) => {
    const source = map.getSource("index") as GeoJSONSource;
    const bounds = map.getBounds();
    const boundPoly: Geometry = {
      "type": "Polygon",
      "coordinates": [
        [
          [bounds.getWest(), bounds.getSouth()],
          [bounds.getEast(), bounds.getSouth()],
          [bounds.getEast(), bounds.getNorth()],
          [bounds.getWest(), bounds.getNorth()],
          [bounds.getWest(), bounds.getSouth()],
        ],
      ],
    };

    const regionCoverer = new geojson.RegionCoverer({
      levelMod: 1,
      maxLevel: S2_BASE_LEVEL,
      minLevel: 0,
      maxCells: 4,
    });
    const cellUnion = regionCoverer.covering(boundPoly);
    const viz = getCellVisualization(cellUnion);
    const cellToks = cellUnion.map((cell) => s2.cellid.toToken(cell));
    // console.log('cellIds:', cellToks);
    source.setData(viz);

    for (const token of cellToks) {
      const resp = await fetch(import.meta.env.VITE_FILES_URL + `/api/v1/catalog/${token}.csv`);
      if (!resp.ok) {
        continue;
      }
      const csv = await resp.text();
      console.log('csv:', csv);
    }

    // const visibleExtents = map.queryRenderedFeatures({
    //   layers: ["index/fill"],
    // });
    // console.log('currently visible:', visibleExtents.map((f) => f.properties?.tileset));
    // const tilesets = new Set(visibleExtents.map((f) => f.properties?.tileset).filter((tileset) => tileset) as string[]);
    // setTilesetHashes((prev) => {
    //   //@ts-expect-error union
    //   if (prev.size === tilesets.size && prev.size === prev.union(tilesets).size) {
    //     return prev;
    //   }
    //   return tilesets;
    // });
  }, []);

  useEffect(() => {
    if (!loadedMap) return;

    refreshVisibleTilesets(loadedMap);
    loadedMap.on('moveend', () => {
      refreshVisibleTilesets(loadedMap);
    });
  }, [refreshVisibleTilesets, loadedMap]);

  useEffect(() => {
    if (!loadedMap) return;

    for (const hash of tilesetHashes) {
      const sourceId = `raster-${hash}`;
      loadedMap.addSource(sourceId, {
        type: "raster",
        url: `pmtiles://${import.meta.env.VITE_FILES_URL}/api/v1/files/${hash}.pmtiles`,
      });
      loadedMap.addLayer({
        id: hash,
        type: "raster",
        source: sourceId,
      });
    }

    return () => {
      for (const tileset of tilesetHashes) {
        loadedMap.removeLayer(tileset);
        loadedMap.removeSource(`raster-${tileset}`);
      }
    };
  }, [loadedMap, tilesetHashes]);

  return (
    <div
      ref={containerRef}
      className="vw-100 vh-100"
    ></div>
  );
};

export default MainMap;
