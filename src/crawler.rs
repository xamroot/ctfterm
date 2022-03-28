use error_chain::error_chain;
use quick_xml::Reader;
use quick_xml::events::Event;
use scraper::{Html, Selector};

error_chain! {
      foreign_links {
          ReqError(reqwest::Error);
          IoError(std::io::Error);
      }
}

pub async fn get_past_events() -> Result<Vec<Vec<String>>> {
	let mut resp = reqwest::get("https://ctftime.org/event/list/past").await.unwrap(); 
	let body = resp.text().await.unwrap();
	let fragment = Html::parse_document(&body);
    let mut ret = vec![];

	let tr_selector = Selector::parse("tr").unwrap();
	let td_selector = Selector::parse("td").unwrap();

	for tr_element in fragment.select(&tr_selector) {
        let mut i:u8 = 0;
        let mut new_event = vec![];
        for td_element in tr_element.select(&td_selector) {
            if i >= 2 {
                break;
            }
            else
            {
                let mut s: String = String::from( td_element.text().collect::<Vec<_>>()[0]);
                // parse date text to fit more easily
                if i > 0 {
                    let comma_idx1: usize = s.match_indices(",").map(|(i, _)|i).collect::<Vec<usize>>()[0];
                    let dash_idx: usize = s.match_indices(" — ").map(|(i, _)|i).collect::<Vec<usize>>()[0];
                    s.replace_range((comma_idx1..dash_idx), "");

                    let comma_idx2: usize = s.match_indices(",").map(|(i, _)|i).collect::<Vec<usize>>()[0];
                    let end_idx: usize = comma_idx2 + 11; 
                    s.replace_range((comma_idx2..end_idx), "");
                }
                new_event.push(s);
            }
            i+=1;
        }
       ret.push(new_event);
	}
   Ok(ret) 
}

pub async fn get_writeups() -> Result<Vec<Vec<String>>> {
	let mut resp = reqwest::get("https://ctftime.org/writeups").await.unwrap(); 
	let body = resp.text().await.unwrap();
	let fragment = Html::parse_document(&body);
    let mut ret = vec![];

	let tr_selector = Selector::parse("tr").unwrap();
	let td_selector = Selector::parse("td").unwrap();

    let tags_idx = 2;

	for tr_element in fragment.select(&tr_selector) {
        let mut new_event = vec![];
        let mut curr_idx = 0;
        for td_element in tr_element.select(&td_selector) {
            // parse new td element
            let elem = td_element.text().collect::<Vec<_>>();
            // ensure elem has an index to be accessed
            if elem.len() > 0
            {
                // check for tags data
                if curr_idx == tags_idx
                {
                    let mut s: String = "".to_string();
                    let mut subindex = 0;
                    while subindex < elem.len()
                    {
                        let new_str = &String::from(elem[subindex]);
                        if new_str.contains("\n\n") == false
                        {
                            s.push_str(new_str);
                        }
                        else
                        {
                            s.push_str(&" ".to_string());
                        }
                        subindex += 1;
                    }
                    new_event.push(s);
                }
                else
                {
                    // grab current data
                    let mut s: String = String::from( elem[0] );
                    new_event.push(s);
                }
            }
            // increase current idx
            // useful for knowing which data we are dealing with
            curr_idx += 1;
        }
        // append new writeup if it's not empty
        if new_event.len() > 0
        {
            ret.push(new_event);
        }
	}
   Ok(ret) 
}

pub async fn get_stats() -> Result<Vec<Vec<String>>> {
	let mut resp = reqwest::get("https://ctftime.org/stats/").await.unwrap(); 
	let body = resp.text().await.unwrap();
	let fragment = Html::parse_document(&body);
    let mut ret = vec![];

	let tr_selector = Selector::parse("tr").unwrap();
	let td_selector = Selector::parse("td").unwrap();

	for tr_element in fragment.select(&tr_selector) {
        let mut new_event = vec![];
        for td_element in tr_element.select(&td_selector) {
            // parse new td element
            let elem = td_element.text().collect::<Vec<_>>();
            // ensure elem has an index to be accessed
            if elem.len() > 0
            {
                let mut s: String = String::from( elem[0] );
                new_event.push(s);
            }
        }
        // append new leaderboard stat if it's not empty
        if new_event.len() > 0
        {
            ret.push(new_event);
        }
	}
   Ok(ret) 
}

pub async fn crawl() -> Result<Vec<String>> {
  let res = reqwest::get("https://ctftime.org/event/list/running/rss/")
    .await?
    .text()
    .await?;

    let mut reader = Reader::from_str(res.as_str());
    reader.trim_text(true);

    let mut count = 0;
    let mut txt = Vec::new();
    let mut buf = Vec::new();

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    loop {
        // NOTE: this is the generic case when we don't know about the input BufRead.
        // when the input is a &str or a &[u8], we don't actually need to use another
        // buffer, we could directly call `reader.read_event_unbuffered()`
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"title" => txt.push(reader.read_text(b"title", &mut Vec::new()).unwrap()),
                    b"tag2" => count += 1,
                    _ => (),
                }
            }
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }


    Ok(txt)
}
