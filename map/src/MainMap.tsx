import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import maplibregl, { GeoJSONSource } from "maplibre-gl";
import * as h3 from "h3-js";
import { area as turfArea } from "@turf/area";
import "maplibre-gl/dist/maplibre-gl.css";
import { Protocol } from "pmtiles";
import { lngLatBoundsToGeoJSON } from "./lib/geo_helpers";

const pmtilesProtocol = new Protocol();
maplibregl.addProtocol("pmtiles", pmtilesProtocol.tile);

const calculateH3Res = (bounds: GeoJSON.Polygon) => {
  const boundsArea = turfArea(bounds);
  for (let res = 9; res >= 0; res--) {
    const cellArea = h3.getHexagonAreaAvg(res, h3.UNITS.m2);
    if (boundsArea / cellArea < 8) {
      return res;
    }
  }
  return 0;
};

const MainMap: React.FC = () => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [loadedMap, setLoadedMap] = useState<maplibregl.Map | undefined>(undefined);
  const [tilesets, setTilesets] = useState<string[]>([]);

  const loadIndex = useCallback(async (map: maplibregl.Map) => {
    const bounds = lngLatBoundsToGeoJSON(map.getBounds());
    const zoom = map.getZoom();
    const res = calculateH3Res(bounds);
    const currentCells = h3.polygonToCells(bounds.coordinates, res, true);
    const cellsWithBuffer = new Set<string>();
    for (const cell of currentCells) {
      cellsWithBuffer.add(cell);
      const neighbors = h3.gridDisk(cell, 1);
      for (const neighbor of neighbors) {
        cellsWithBuffer.add(neighbor);
      }
    }
    console.log(`Using H3 res ${res} for zoom=${zoom}, ${currentCells.length} cells, ${cellsWithBuffer.size} cells with buffer`);

    const src = map.getSource("index") as GeoJSONSource;
    src.updateData({ removeAll: true });
    const loadIndexes: Promise<void>[] = [];
    const tilesetIds: Set<string> = new Set();
    for (const cell of cellsWithBuffer) {
      const cellGeom = h3.cellToBoundary(cell, true);
      const cellFeature: GeoJSON.Feature = {
        type: "Feature",
        id: cell,
        properties: {
          cell,
        },
        geometry: {
          type: "Polygon",
          coordinates: [cellGeom],
        },
      };
      src.updateData({
        add: [cellFeature],
      });
      loadIndexes.push((async () => {
        const index = await fetch(`${import.meta.env.VITE_FILES_URL}/api/v1/index/${cell}.json`);
        if (index.status !== 200) { return; }
        const indexJson = await index.json();
        src.updateData({
          update: [
            {
              id: cell,
              addOrUpdateProperties: [
                { key: "count", value: indexJson.count },
              ]
            },
          ],
        });
        for (const tilesetId of indexJson.tilesets) {
          tilesetIds.add(tilesetId);
        }
      })());
    }
    await Promise.all(loadIndexes);
    console.log(`Detected ${tilesetIds.size} tilesets`);
    setTilesets(Array.from(tilesetIds));
  }, []);

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

  useEffect(() => {
    if (!loadedMap) return;

    loadIndex(loadedMap);
    loadedMap.on('moveend', () => {
      loadIndex(loadedMap);
    });
  }, [loadIndex, loadedMap]);

  useEffect(() => {
    if (!loadedMap) return;

    for (const tileset of tilesets) {
      const sourceId = `raster-${tileset}`;
      loadedMap.addSource(sourceId, {
        type: "raster",
        url: `pmtiles://${import.meta.env.VITE_FILES_URL}/api/v1/tiles/${tileset}.pmtiles`,
      });
      loadedMap.addLayer({
        id: tileset,
        type: "raster",
        source: sourceId,
      });
    }

    return () => {
      for (const tileset of tilesets) {
        loadedMap.removeLayer(tileset);
        loadedMap.removeSource(`raster-${tileset}`);
      }
    };
  }, [loadedMap, tilesets]);

  return (
    <div
      ref={containerRef}
      className="vw-100 vh-100"
    ></div>
  );
};

export default MainMap;
