import { LngLatBounds } from 'maplibre-gl';

// Function to convert LngLatBounds to GeoJSON Polygon geometry
export function lngLatBoundsToGeoJSON(bounds: LngLatBounds): GeoJSON.Polygon {
  // Extract southwest and northeast coordinates from bounds
  const sw = bounds.getSouthWest();
  const ne = bounds.getNorthEast();

  // Create the four corners of the bounding box
  const coordinates: [number, number][] = [
    [sw.lng, sw.lat], // Southwest corner
    [ne.lng, sw.lat], // Southeast corner
    [ne.lng, ne.lat], // Northeast corner
    [sw.lng, ne.lat], // Northwest corner
    [sw.lng, sw.lat], // Close the polygon by returning to the Southwest corner
  ];

  // Return the GeoJSON polygon object
  return {
    type: "Polygon",
    coordinates: [coordinates]
  };
}
