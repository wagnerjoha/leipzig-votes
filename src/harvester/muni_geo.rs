use anyhow::{anyhow, Result};
use geo::Polygon;
use geojson::GeoJson;
use reqwest::Client;

use std::collections::HashMap;

pub async fn fetch_geom(client: &Client) -> Result<HashMap<String, Polygon<f64>>> {
    let url = "https://static.leipzig.de/fileadmin/mediendatenbank/leipzig-de/Stadt/02.1_Dez1_Allgemeine_Verwaltung/12_Statistik_und_Wahlen/Geodaten/Ortsteile_Leipzig_UTM33N.json";

    let response = client.get(url).send().await?.text().await?;
    let geojson = response.parse::<GeoJson>()?;

    let mut geometry_map: HashMap<String, Polygon<f64>> = HashMap::new();

    if let GeoJson::FeatureCollection(collection) = geojson {
        for feature in collection.features {
            if let (Some(properties), Some(geometry)) = (&feature.properties, &feature.geometry) {
                if let Some(name) = properties.get("Name").and_then(|name| name.as_str()) {
                    if let Ok(polygon) = geometry.value.clone().try_into() {
                        geometry_map.insert(name.to_string(), polygon);
                    }
                }
            }
        }
        Ok(geometry_map)
    } else {
        Err(anyhow!(
            "Expected a FeatureCollection, but got something else"
        ))
    }
}
