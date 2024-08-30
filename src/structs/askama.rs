use std::collections::HashMap;

use anyhow::Result;
use askama::Template;

use crate::structs::votes::Vote;

#[derive(Template)]
#[template(path = "descriptives.html")]
pub struct VoteSummaryTemplate<'a> {
    vote_name: &'a str,
    vote_records: Vec<VoteRecordTemplate<'a>>,
}

pub struct VoteRecordTemplate<'a> {
    name_muni: &'a str,
    vote_perc: &'a HashMap<String, f64>,
    total_votes: &'a i32,
    _has_geometry: bool,
}

pub fn render_html_summary(vote: &Vote) -> Result<String> {
    let vote_records: Vec<VoteRecordTemplate> = vote
        .vote_records
        .iter()
        .map(|record| VoteRecordTemplate {
            name_muni: &record.name_muni,
            vote_perc: &record.vote_perc,
            total_votes: &record.total_votes,
            _has_geometry: record.geometry.is_some(),
        })
        .collect();

    let template = VoteSummaryTemplate {
        vote_name: &vote.name,
        vote_records,
    };

    Ok(template.render()?)
}
