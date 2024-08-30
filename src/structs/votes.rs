use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, Context, Error, Result};
use geo::algorithm::proj::Proj;
use geo::{Coord, LineString, Polygon};
use geojson::{
    Feature, FeatureCollection, GeoJson, Geometry as GeoJsonGeom, Value as GeoJsonValue,
};
use serde_json::{Number, Value as JsonValue};

#[derive(Debug)]
pub struct Vote {
    pub name: String,
    pub vote_records: Vec<VoteRecord>,
}

#[derive(Debug)]
pub struct VoteRecord {
    pub name_muni: String,
    pub votes: HashMap<String, i32>,
    pub geometry: Option<Polygon<f64>>,
    pub total_votes: i32,
    pub vote_perc: HashMap<String, f64>,
}

impl Vote {
    pub fn write_geojson(&self) -> Result<()> {
        let filename = self.name.as_str();

        let features: Vec<Feature> = self
            .vote_records
            .iter()
            .map(|record| {
                // Create properties
                let mut properties = serde_json::Map::new();
                properties.insert(
                    "name_muni".to_string(),
                    JsonValue::String(record.name_muni.clone()),
                );
                properties.insert(
                    "total_votes".to_string(),
                    JsonValue::Number(record.total_votes.into()),
                );

                // Add vote percentages to properties
                for (party, count) in &record.vote_perc {
                    let rounded = (count * 100.0).round() / 100.0;
                    if let Some(num) = Number::from_f64(rounded) {
                        properties.insert(party.clone(), JsonValue::Number(num));
                    } else {
                        // Fallback in case the f64 can't be exactly represented as a JSON number
                        properties.insert(
                            party.clone(),
                            JsonValue::Number(Number::from_f64(count.round()).unwrap()),
                        );
                    }
                }

                // Add absolute vote counts to properties
                for (party, count) in &record.votes {
                    let num = serde_json::Number::from(*count as i64);
                    properties.insert(format!("{}_absolut", party), JsonValue::Number(num));
                }

                // Create geometry
                let geometry = record.geometry.as_ref().map(|polygon| {
                    GeoJsonGeom::new(geojson::Value::Polygon(vec![polygon
                        .exterior()
                        .coords()
                        .map(|coord| vec![coord.x, coord.y])
                        .collect()]))
                });

                // Create feature
                Feature {
                    bbox: None,
                    geometry,
                    id: None,
                    properties: Some(properties),
                    foreign_members: None,
                }
            })
            .collect();

        let feature_collection = FeatureCollection {
            bbox: None,
            features,
            foreign_members: None,
        };

        // Convert FeatureCollection to GeoJson
        let geojson = GeoJson::FeatureCollection(feature_collection);

        // Serialize to a JSON string
        let geojson_string = serde_json::to_string_pretty(&geojson)?;

        // Write to a file
        // let path = "./data/".to_string() + filename + ".json";
        let path = format!("./data/{}.json", filename);
        let mut file = File::create(path)?;
        file.write_all(geojson_string.as_bytes())
            .expect("Failed to write GeoJson");

        Ok(())
    }

    pub fn from_geojson(filename: &str) -> Result<Self> {
        let data_dir = "data";
        let file_path = Path::new(data_dir).join(format!("{}.json", &filename));

        let geojson_str = read_to_string(&file_path)
            .with_context(|| format!("Failed to read GeoJSON file: {}", file_path.display()))?;

        let geojson: GeoJson = geojson_str.parse().with_context(|| {
            format!("Failed to parse GeoJSON from file: {}", file_path.display())
        })?;

        let mut vote_records = Vec::new();

        if let GeoJson::FeatureCollection(collection) = geojson {
            for feature in collection.features {
                if let Some(record) = Self::parse_feature(feature) {
                    vote_records.push(record);
                }
            }
        }

        Ok(Vote {
            name: filename.to_string(),
            vote_records,
        })
    }

    fn parse_feature(feature: Feature) -> Option<VoteRecord> {
        let properties = feature.properties?;
        let name_muni = properties.get("name_muni")?.as_str()?;

        let geom_json = feature.geometry.unwrap();

        let polygon = match geom_json.value {
            GeoJsonValue::Polygon(coords) => {
                let exterior: Vec<Coord<f64>> = coords
                    .first()?
                    .iter()
                    .map(|c| Coord { x: c[0], y: c[1] })
                    .collect();

                let interiors: Vec<LineString<f64>> = coords
                    .iter()
                    .skip(1)
                    .map(|ring| ring.iter().map(|c| Coord { x: c[0], y: c[1] }).collect())
                    .collect();

                Some(Polygon::new(exterior.into(), interiors))
            }
            _ => None,
        };

        let mut votes = HashMap::new();
        for (key, value) in properties.iter() {
            if key != "name_muni" && key != "total_votes" {
                if let Some(count) = value.as_i64() {
                    votes.insert(key.clone(), count as i32);
                }
            }
        }

        Some(VoteRecord::new(name_muni.to_string(), votes, polygon))
    }

    pub fn convert_wgs84(&self) -> Result<Self> {
        let from = "EPSG:32633";
        let to = "EPSG:4326";

        let converted_records: Vec<_> = self
            .vote_records
            .iter()
            .map(|record| {
                let converted_polygon = match &record.geometry {
                    Some(polygon) => {
                        // Convert exterior
                        let converted_exterior: Result<LineString<f64>, Error> = polygon
                            .exterior()
                            .coords()
                            .map(|coord| reproject_coord_wgs84(*coord, from, to))
                            .collect::<Result<Vec<_>>>()
                            .map(LineString::new);

                        // Convert interiors
                        let converted_interiors: Result<Vec<LineString<f64>>> = polygon
                            .interiors()
                            .iter()
                            .map(|line| {
                                line.coords()
                                    .map(|coord| reproject_coord_wgs84(*coord, from, to))
                                    .collect::<Result<Vec<_>>>()
                                    .map(LineString::new)
                            })
                            .collect();

                        match (converted_exterior, converted_interiors) {
                            (Ok(ext), Ok(int)) => Ok(Some(Polygon::new(ext, int))),
                            _ => Err(anyhow!("Failed to convert coordinates")),
                        }
                    }
                    None => Ok(None),
                };

                VoteRecord::new(
                    record.name_muni.clone(),
                    record.votes.clone(),
                    converted_polygon.unwrap(),
                )
            })
            .collect();

        Ok(Vote {
            name: self.name.clone(),
            vote_records: converted_records,
        })
    }
}

impl VoteRecord {
    pub fn new(
        name_muni: String,
        votes: HashMap<String, i32>,
        geometry: Option<Polygon<f64>>,
    ) -> Self {
        let total_votes = votes.values().sum();
        let vote_perc = Self::calc_perc(&votes, total_votes);

        VoteRecord {
            name_muni,
            votes,
            geometry,
            total_votes,
            vote_perc,
        }
    }

    fn calc_perc(votes: &HashMap<String, i32>, total_votes: i32) -> HashMap<String, f64> {
        votes
            .iter()
            .map(|(party, count)| {
                let perc = (*count as f64 / total_votes as f64) * 100.0;
                (party.clone(), perc)
            })
            .collect()
    }
}

fn reproject_coord_wgs84(coord: Coord<f64>, from: &str, to: &str) -> Result<Coord<f64>> {
    let ft_to_m = Proj::new_known_crs(from, to, None).unwrap();

    let result = ft_to_m
        .convert(Coord {
            x: coord.x,
            y: coord.y,
        })
        .unwrap();

    Ok(Coord {
        x: result.x,
        y: result.y,
    })
}
