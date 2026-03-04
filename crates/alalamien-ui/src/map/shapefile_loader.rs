use bevy::prelude::Resource;
use shapefile::{PolygonRing, Shape};
use shapefile::dbase::FieldValue;
use std::path::Path;

use crate::map::projection::{lon_to_x, lat_to_y};

/// A single polygon ring expressed in world-space (projected) coordinates.
/// `is_hole == true` means this ring is an interior boundary (enclave / lake).
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RingData {
    /// Projected (x, y) vertices in world space.
    pub vertices: Vec<(f32, f32)>,
    pub is_hole: bool,
}

/// One logical polygon group: an outer ring plus zero or more inner holes.
#[derive(Clone, Debug)]
pub struct PolygonGroupData {
    pub outer: Vec<(f32, f32)>,
    pub holes: Vec<Vec<(f32, f32)>>,
}

/// All geographic data for a single nation (may have multiple polygon groups,
/// e.g. archipelagos or exclave-heavy borders like Russia / USA).
#[derive(Clone, Debug)]
pub struct NationGeoData {
    pub name: String,
    pub iso_a3: String,
    #[allow(dead_code)]
    pub iso_a2: String,
    /// Polygon groups — each will become its own tessellated mesh.
    pub groups: Vec<PolygonGroupData>,
}

/// Bevy resource holding the full set of loaded nation geometries.
#[derive(Resource, Default)]
pub struct WorldGeoData {
    pub nations: Vec<NationGeoData>,
}

/// Load `ne_110m_admin_0_countries.shp` (and its companion `.dbf`) from `shp_path`.
/// Returns parsed [`WorldGeoData`].
pub fn load_countries_shp(shp_path: &Path) -> Result<WorldGeoData, String> {
    let mut reader =
        shapefile::Reader::from_path(shp_path).map_err(|e| format!("Cannot open shapefile: {e}"))?;

    let mut nations: Vec<NationGeoData> = Vec::new();

    for result in reader.iter_shapes_and_records() {
        let (shape, record) = match result {
            Ok(pair) => pair,
            Err(e) => {
                tracing::warn!("Skipping bad record: {e}");
                continue;
            }
        };

        // Extract name and ISO codes from the DBF record.
        let name = field_str(&record, "ADMIN")
            .or_else(|| field_str(&record, "NAME"))
            .unwrap_or_else(|| "Unknown".to_string());
        let iso_a3 = field_str(&record, "ADM0_A3")
            .or_else(|| field_str(&record, "ISO_A3"))
            .unwrap_or_else(|| "UNK".to_string());
        let iso_a2 = field_str(&record, "ISO_A2")
            .or_else(|| field_str(&record, "GU_A3"))
            .unwrap_or_else(|| "--".to_string());

        let groups = match shape {
            Shape::Polygon(poly) => rings_to_groups(poly.rings()),
            Shape::PolygonM(poly) => {
                let rings: Vec<PolygonRing<shapefile::PointM>> = poly.into_inner();
                // Convert PointM → (lon, lat)
                let converted: Vec<PolygonRing<shapefile::Point>> = rings
                    .into_iter()
                    .map(|r| match r {
                        PolygonRing::Outer(pts) => PolygonRing::Outer(
                            pts.into_iter()
                                .map(|p| shapefile::Point { x: p.x, y: p.y })
                                .collect(),
                        ),
                        PolygonRing::Inner(pts) => PolygonRing::Inner(
                            pts.into_iter()
                                .map(|p| shapefile::Point { x: p.x, y: p.y })
                                .collect(),
                        ),
                    })
                    .collect();
                rings_to_groups(&converted)
            }
            Shape::PolygonZ(poly) => {
                let rings: Vec<PolygonRing<shapefile::PointZ>> = poly.into_inner();
                let converted: Vec<PolygonRing<shapefile::Point>> = rings
                    .into_iter()
                    .map(|r| match r {
                        PolygonRing::Outer(pts) => PolygonRing::Outer(
                            pts.into_iter()
                                .map(|p| shapefile::Point { x: p.x, y: p.y })
                                .collect(),
                        ),
                        PolygonRing::Inner(pts) => PolygonRing::Inner(
                            pts.into_iter()
                                .map(|p| shapefile::Point { x: p.x, y: p.y })
                                .collect(),
                        ),
                    })
                    .collect();
                rings_to_groups(&converted)
            }
            _ => {
                tracing::debug!("Skipping non-polygon shape for nation '{name}'");
                continue;
            }
        };

        if groups.is_empty() {
            continue;
        }

        nations.push(NationGeoData {
            name,
            iso_a3,
            iso_a2,
            groups,
        });
    }

    tracing::info!("Loaded {} nations from shapefile", nations.len());
    Ok(WorldGeoData { nations })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Group a slice of `PolygonRing<Point>` into logically-associated
/// (outer + holes) pairs and project each point to world space.
fn rings_to_groups(rings: &[PolygonRing<shapefile::Point>]) -> Vec<PolygonGroupData> {
    let mut groups: Vec<PolygonGroupData> = Vec::new();

    for ring in rings {
        match ring {
            PolygonRing::Outer(pts) => {
                if pts.len() < 3 {
                    continue;
                }
                groups.push(PolygonGroupData {
                    outer: project_points(pts),
                    holes: Vec::new(),
                });
            }
            PolygonRing::Inner(pts) => {
                if pts.len() < 3 {
                    continue;
                }
                if let Some(last) = groups.last_mut() {
                    last.holes.push(project_points(pts));
                }
            }
        }
    }

    groups
}

/// Project a slice of `shapefile::Point` (lon/lat) to world-space (x, y).
fn project_points(pts: &[shapefile::Point]) -> Vec<(f32, f32)> {
    pts.iter()
        .map(|p| (lon_to_x(p.x), lat_to_y(p.y)))
        .collect()
}

/// Extract a string field from a DBF record, trimming whitespace.
fn field_str(record: &shapefile::dbase::Record, key: &str) -> Option<String> {
    match record.get(key)? {
        FieldValue::Character(Some(s)) => {
            let s = s.trim().to_string();
            if s.is_empty() || s == "-99" || s == "-1" { None } else { Some(s) }
        }
        _ => None,
    }
}
