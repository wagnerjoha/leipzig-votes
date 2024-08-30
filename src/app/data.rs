use std::fs;
use std::time::Instant;

use actix_web::HttpResponse;
use anyhow::Result;

use crate::harvester::load_config;
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

    let mut votes = Vec::new();

    for source in config.vote_sources {
        dbg!(&source.name);
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
