use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use actix_web::HttpResponse;
use anyhow::{Context, Result};
use reqwest::Client;

use crate::harvester::load_config;
use crate::harvester::muni_geo::fetch_geom;
use crate::harvester::votes_lpz::harvest_votes;
use crate::structs::askama::render_html_summary;
use crate::structs::votes::Vote;

pub async fn data_items() -> HttpResponse {
    println!("Data Items function started.");
    let votes = match get_data().await {
        Ok(votes) => votes,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get data"),
    };

    let html_summary = match render_html_summary(&votes[0]) {
        Ok(result) => result,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to render Html"),
    };

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_summary)
}

pub async fn get_data() -> Result<Vec<Vote>> {
    let start = Instant::now();
    println!("Instant started.");

    let config = load_config()?;

    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;

    let mut votes = Vec::new();

    let geom_map = fetch_geom(&client).await?;

    for source in config.vote_sources {
        dbg!(&source.name);
        let data_dir = "data";
        let path = PathBuf::from(data_dir).join(format!("{}.json", &source.name));
        if !path.exists() {
            std::fs::create_dir_all(data_dir)?;
            println!("Data harvest starts for {}", &source.name);
            let vote = harvest_votes(
                &client,
                &source.url,
                &source.name,
                &source.party_map,
                &geom_map,
            )
            .await?;
            let vote_wgs84 = vote.convert_wgs84().unwrap();

            let _ = vote_wgs84
                .write_geojson()
                .context("Failed to write GeoJson.");
        }
        println!("Read started.");
        let vote = Vote::from_geojson(&source.name)?;
        votes.push(vote);
    }

    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);

    Ok(votes)
}

pub fn read_file(file_path: &str) -> String {
    fs::read_to_string(file_path).expect("Could not find html file for page rendering")
}
