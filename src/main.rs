use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span,Spans},
    widgets::{List, ListItem, ListState, Block, BorderType, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};
use tokio::runtime::Runtime;
use tokio::time::*;
use tokio::task;
use tokio::sync::Mutex;
use futures::executor::block_on;
mod loaders;
mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // initialize app
    let mut app = types::App::new();

    let res = run_app(&mut terminal, &mut app).await;
    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    /*if let Err(err) = res {
        println!("{:?}", err)
    }*/

    Ok(())
}

async fn run_app<'a,B: Backend>(terminal: &mut Terminal<B>, app: &'a mut types::App<'a>) -> io::Result<()> {
	let up_input = std::sync::Mutex::new(0);
    let up = std::sync::Arc::new(up_input);
    let inputThreadUp = up.clone();

	let down_input = std::sync::Mutex::new(0);
    let down = std::sync::Arc::new(down_input);
    let inputThreadDown = down.clone();

	let right_input = std::sync::Mutex::new(0);
    let right = std::sync::Arc::new(right_input);
    let inputThreadRight = right.clone();

	let left_input = std::sync::Mutex::new(0);
    let left = std::sync::Arc::new(left_input);
    let inputThreadLeft = left.clone();

	let x = std::sync::Mutex::new(0);
	let arc = std::sync::Arc::new(x);	
	let inputThreadRunning = arc.clone();
    let dataThreadRunning = arc.clone();


	let mut scrollCounter = 0;
	let scrollTimer = 100;
	let mut autoscroll = true;


	let loaded = std::sync::Arc::new( std::sync::Mutex::new(0) );
	let loaded1_ = loaded.clone();
	let loaded2_ = loaded.clone();
	let loaded3_ = loaded.clone();
	let loaded4_ = loaded.clone();
    let load_total = 4;

	let needs_load = std::sync::Arc::new( std::sync::Mutex::new(true) );
    let needs_load_ = needs_load.clone();

    let results =  tokio::join!(
                    tokio::spawn( 
                            async move {
                            let mut curr_events = vec![];
                            loaders::load_current_events(&mut curr_events).await;
                            *loaded1_.lock().unwrap() += 1;
                            return curr_events;
                        }
                    ),
                    tokio::spawn( 
                        async move {
                            let mut stats = vec![];
                            loaders::load_leaderboard(&mut stats).await;
                            *loaded2_.lock().unwrap() += 1;
                            return stats;
                        }
                    ),
                    tokio::spawn( 
                        async move {
                            let mut past_events = vec![];
                            loaders::load_past_events(&mut past_events).await;
                            *loaded3_.lock().unwrap() += 1;
                            return past_events;
                        }
                    ),
                    tokio::spawn( 
                        async move {
                            let mut writeups = vec![];
                            loaders::load_writeups(&mut writeups).await;
                            *loaded4_.lock().unwrap() += 1;
                            return writeups;
                        }
                    ));
    let results_ = std::sync::Arc::new( std::sync::Mutex::new( results));

	// create input handler thread
    let inputThreadHandle = std::thread::spawn(move || {
		while *inputThreadRunning.lock().unwrap() < 1 {
			if let Event::Key(key) = event::read().unwrap() {
				if let KeyCode::Char('q') = key.code {
					*inputThreadRunning.lock().unwrap() = 1;
				}
				else if let KeyCode::Char('w') = key.code {
					*inputThreadUp.lock().unwrap() = 1;
				}
				else if let KeyCode::Char('s') = key.code {
					*inputThreadDown.lock().unwrap() = 1;
				}
				else if let KeyCode::Char('d') = key.code {
					*inputThreadRight.lock().unwrap() = 1;
				}
				else if let KeyCode::Char('a') = key.code {
					*inputThreadLeft.lock().unwrap() = 1;
				}
			}
		}
    });

	let mut running = 0;
    while running < 1 {
		// updated shared running var
		running = *arc.lock().unwrap();

        // handle inputs
        if *down.lock().unwrap() > 0
        {
            if app.focused < 1
            {
                app.focused = 4;
            }
            else
            {
                app.focused -= 1;
            }
            *down.lock().unwrap() = 0;
        }
        else if *up.lock().unwrap() > 0
        {
            app.focused = (app.focused+1)%5;
            *up.lock().unwrap() = 0;
        }
        else if *right.lock().unwrap() > 0
        {
            if (app.focused == 3)
            {
               app.writeups.move_down(); 
            }
            else if (app.focused == 4)
            {
               app.leaderboard_stats.move_down(); 
            }
            *right.lock().unwrap() = 0;
        }
        else if *left.lock().unwrap() > 0
        {
            if (app.focused == 3)
            {
               app.writeups.move_up(); 
            }
            else if (app.focused == 4)
            {
                app.leaderboard_stats.move_up();
            }
            *left.lock().unwrap() = 0;
        }

		// handle data auto-scrolling
		if autoscroll
		{
			// check timer, for artificial sleep
			// using sleep would get funky with input
			if scrollCounter <= 0
			{
				scrollCounter = scrollTimer;

                // scroll current event string
                if app.curr_events.items.len() > 0
                {
                    let mut curr_event_s = String::new();
                    let mut first_ch = String::new();

                    curr_event_s = app.curr_events.get(0).clone();

                    first_ch.push(curr_event_s.chars().nth(0).unwrap());

                    curr_event_s = curr_event_s[1..curr_event_s.len()].to_string();
                    curr_event_s.push_str(&first_ch);

                    app.curr_events.set(curr_event_s, 0);
                }
			}
			else
			{
				scrollCounter -= 1;
			}
		}

		// update terminal view
        terminal.draw(|f| ui(f, app));
	    
        let tmp_needs_load = *needs_load.lock().unwrap();
        if tmp_needs_load
        {
            if *loaded.lock().unwrap() == load_total
            {
                let res = &*results_.lock().unwrap();
                let new_items = &res;
                app.curr_events.update( &new_items.0.as_ref().unwrap() );
                app.leaderboard_stats.update( &new_items.1.as_ref().unwrap() );
                app.past_events_list.update( &new_items.2.as_ref().unwrap() );
                app.writeups.update( &new_items.3.as_ref().unwrap() );
            }
        }
    }

	inputThreadHandle.join().unwrap();
    Ok(())
}


/*
 * build_leaderboard
 * create leaderboard widget via App.leaderboard_stats 
 */
fn build_leaderboard<'a>(app:&'a mut types::App) -> Table<'a> {
    let mut color: Color = Color::Red; 
    if app.focused == 4
    {
        color = Color::White;
    }
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().fg(Color::Red);

    // create surrounding block
    let block = Block::default()
        .title(Span::styled(
            "Leaderboard",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title_alignment(Alignment::Left);
    
    // set up headers
    let header_cells = [" ", " ", " ", " "]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(0);

    // create rows of leaderboard data
    let rows = app.leaderboard_stats.items.iter().map(|(a,b,c,d)| {
        let height = 1;
        let mut cells = vec![];
        cells.push( Cell::from( String::from(a.clone()) ) );
        cells.push( Cell::from( String::from(b.clone()) ) );
        cells.push( Cell::from( String::from(c.clone()) ) );
        cells.push( Cell::from( String::from(d.clone()) ) );
        Row::new(cells).height(height as u16)
    });

    // render data into table
    let t = Table::new(rows)
        .header(header)
        .block(block)
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(15),
            Constraint::Percentage(40),
            Constraint::Percentage(10),
            Constraint::Percentage(35),
        ]);
    return t;
}

fn build_current_events<'a>(app :&'a mut types::App, width: usize) -> List<'a> {
    let mut color: Color = Color::Red; 
    if app.focused == 1
    {
        color = Color::White;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(Span::styled(
            "Now Running",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Right);
    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .curr_events
        .items
        .iter()
        .map(|name| {
            ListItem::new(vec![
                Spans::from("-".repeat(width)),
                Spans::from(vec![Span::from(Span::styled(name, Style::default().add_modifier(Modifier::BOLD)))]),
            ])
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    return items;
}

fn build_past_events<'a>(app :&'a mut types::App, width: usize) -> List<'a> {
    let mut color: Color = Color::Red; 
    if app.focused == 2
    {
        color = Color::White;
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(Span::styled(
            "Past Events",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Right);
    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .past_events_list
        .items
        .iter()
        .map(|(name, data)| {
            ListItem::new(vec![
                Spans::from("-".repeat(width)),
                Spans::from(vec![Span::from(Span::styled(name, Style::default().add_modifier(Modifier::BOLD)))]),
                Spans::from(vec![Span::raw(data)]),
            ])
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    return items;
}

fn build_writeups<'a>(app :&'a mut types::App, width: usize) -> List<'a> {
    // NOTE: each entry has a height 6.5
    let mut color: Color = Color::Red; 
    if app.focused == 3
    {
        color = Color::White;
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(Span::styled(
            "Write Ups",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Right);
    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .writeups
        .items
        .iter()
        .map(|(w1, w2, w3, w4, w5)| {
            ListItem::new(vec![
                Spans::from(vec![
                            Span::raw("["), Span::raw(w3), Span::raw("]"),
                            /*Span::raw(w1), Span::raw(" "), */Span::raw(w2),]),
            ])
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    return items;
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut types::App) {
    // Wrapping block for a group
    // Just draw the block and the group on the same area and build the group
    // with at least a margin of 1
    let size = f.size();

    // Surrounding block
    let block = Block::default()
        .title("CTF>TERM")
        .title_alignment(Alignment::Left)
        .borders(Borders::TOP);
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
        .split(f.size());

    // Top two inner blocks
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(chunks[0]);

    let top_left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(top_chunks[0]);

    // build widgets
	f.render_widget(build_leaderboard(app), top_left_chunks[1]);
    f.render_widget(build_current_events(app, top_left_chunks[0].width as usize), top_left_chunks[0]);

    // build past events widget
    f.render_widget(build_past_events(app, top_chunks[1].width as usize), top_chunks[1]);

    // Bottom two inner blocks
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    // Bottom left block with all default borders
    f.render_widget(build_writeups(app, bottom_chunks[0].width as usize), bottom_chunks[0]);

    // Bottom right block with styled left and right border
    // build calender
    let block = Block::default()
        .title(Span::styled(
            "Write Up Watchlist",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(Color::Red))
        .borders(Borders::ALL);
    f.render_widget(block, bottom_chunks[1]);
}

