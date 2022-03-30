use std::{error::Error, io};
use futures::executor::block_on;
// just 'mod crawler' doesnt work for whatever
// fucking reason >:I
#[path = "crawler.rs"] mod crawler;

pub async fn load_past_events(mut past_events_items: &mut Vec<(String, String)>)->io::Result<()>
{
    // get past events
    let past_evts: Vec<Vec<String>> = crawler::get_past_events().await.unwrap();
    for evt in &past_evts {
        if evt.len() > 0 {
            past_events_items.push((evt[0].clone(), evt[1].clone()));
        }
    }
    Ok(())
}

pub async fn load_leaderboard(mut leaderboard_items: &mut Vec<(String, String, String, String)>)->io::Result<()>
{
    // get leaderboard stats
    let leaderboard_stats: Vec<Vec<String>> = crawler::get_stats().await.unwrap();
    // update app with leaderboard stats
    for stat in &leaderboard_stats {
        leaderboard_items.push(
            (
                stat[0].clone(), 
                stat[1].clone(), 
                stat[3].clone(), 
                stat[2].clone()
            )
        );
    }
    Ok(())
}

pub async fn load_current_events(mut current_events_items: &mut Vec<String>)->io::Result<()>
{
    // get current events
    let evts: Vec<String> = crawler::crawl().await.unwrap();

    // update app with running events
    let mut i: usize = 0;
    for evt in &evts{
        if i > 0 {
            for ch in evts[i].chars()
            {
                current_events_items.push(evts[i].clone());
            }
        }
        i += 1;
    }
    
    if current_events_items.len() < 1
    {
        current_events_items.push(String::from("None"));
    }

    Ok(())
}

pub async fn load_writeups(mut writeup_items: &mut Vec<(String,String,String,String,String)>)->io::Result<()>
{
    // get write ups
    let writeups: Vec<Vec<String>> = crawler::get_writeups().await.unwrap();
    // update app with writeups
    for writeup in &writeups {
        writeup_items.push(
            (
                writeup[0].clone(), 
                writeup[1].clone(), 
                writeup[2].clone(), 
                writeup[3].clone(), 
                writeup[4].clone()
            )
        );
    }

    Ok(())
}
