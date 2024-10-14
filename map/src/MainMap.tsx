import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import maplibregl from "maplibre-gl";
import "maplibre-gl/dist/maplibre-gl.css";
import { Protocol } from "pmtiles";

const pmtilesProtocol = new Protocol();
maplibregl.addProtocol("pmtiles", pmtilesProtocol.tile);
maplibregl.addProtocol("oom-files", async (params, abort) => {
  const prefix = import.meta.env.VITE_FILES_URL;
  const inputUrl = new URL(params.url);
  const pathname = inputUrl.pathname.replace(/^\/+/, "");
  const url = `${prefix}/${pathname}`;
  const response = await fetch(url, { signal: abort.signal });
  if (!response.ok) {
    throw new Error(`Failed to fetch ${url}: ${response.statusText}`);
  }
  if (params.type === "json") {
    let text = await response.text();
    text = text.replace(/"oom-files:\/\/\//g, `"${prefix}/`);
    return {
      data: JSON.parse(text),
    };
  } else if (params.type === "string") {
    return {
      data: await response.text(),
    };
  } else if (params.type === "arrayBuffer") {
    return {
      data: await response.arrayBuffer(),
    };
  } else {
    throw new Error(`Unknown type: ${params.type}`);
  }
});

const MainMap: React.FC = () => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [loadedMap, setLoadedMap] = useState<maplibregl.Map | undefined>(undefined);
  const [tilesets, setTilesets] = useState<Set<string>>(new Set());

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

  const refreshVisibleTilesets = useCallback((map: maplibregl.Map) => {
    const visibleExtents = map.queryRenderedFeatures({
      layers: ["index/fill"],
    });
    console.log('currently visible:', visibleExtents.map((f) => f.properties?.tileset));
    const tilesets = new Set(visibleExtents.map((f) => f.properties?.tileset).filter((tileset) => tileset) as string[]);
    setTilesets((prev) => {
      //@ts-ignore
      if (prev.size === tilesets.size && prev.size === prev.union(tilesets).size) {
        return prev;
      }
      return tilesets;
    });
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
