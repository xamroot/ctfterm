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
use futures::executor::block_on;
mod loaders;

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
    idx: usize
}

impl<T: Clone> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
            idx: 0,
        }
    }
	
    fn scroll(&mut self) {
        self.items.push(self.items[0].clone());
        self.items.remove(0);
    }

    fn move_down(&mut self){
        if self.idx < self.items.len()-1
        {
            self.items.push(self.items[0].clone());
            self.items.remove(0);
            self.idx += 1;
        }
    }


    // a b c d e f
    // b c d e f a
    // c d e f a b
    // b c d e f a
    fn move_up(&mut self){
        if self.idx > 0
        {
            self.items.insert(0, self.items[self.items.len()-1].clone());
            self.items.remove(self.items.len()-1);
            self.idx -= 1;
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct App<'a> {
    focused: i16,
    term_height: u16,
    curr_events: StatefulList<String>,
    current_events_list: StatefulList<String>,
    past_events_list: StatefulList<(String, String)>,
    leaderboard_stats: StatefulList<(String, String, String, String)>,
    writeups: StatefulList<(String, String, String, String, String)>,
    events: Vec<(&'a str, &'a str)>,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            focused: 0,
            term_height: 0,
            curr_events: StatefulList::with_items(vec![
            ]),
            current_events_list: StatefulList::with_items(vec![
            ]),
            past_events_list: StatefulList::with_items(vec![
            ]),
            leaderboard_stats: StatefulList::with_items(vec![
            ]),
            writeups: StatefulList::with_items(vec![
            ]),
            events: vec![
                ("Event1", "INFO"),
                ("Event2", "INFO"),
                ("Event3", "CRITICAL"),
                ("Event4", "ERROR"),
                ("Event5", "INFO"),
                ("Event6", "INFO"),
                ("Event7", "WARNING"),
                ("Event8", "INFO"),
                ("Event9", "INFO"),
                ("Event10", "INFO"),
                ("Event11", "CRITICAL"),
                ("Event12", "INFO"),
                ("Event13", "INFO"),
                ("Event14", "INFO"),
                ("Event15", "INFO"),
                ("Event16", "INFO"),
                ("Event17", "ERROR"),
                ("Event18", "ERROR"),
                ("Event19", "INFO"),
                ("Event20", "INFO"),
                ("Event21", "WARNING"),
                ("Event22", "INFO"),
                ("Event23", "INFO"),
                ("Event24", "WARNING"),
                ("Event25", "INFO"),
                ("Event26", "INFO"),
            ],
        }
    }

    /// Rotate through the event list.
    /// This only exists to simulate some kind of "progress"
    fn on_tick(&mut self) {
        let event = self.events.remove(0);
        self.events.push(event);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // initialize app
    let mut app = App::new();

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

async fn run_app<'a,B: Backend>(terminal: &mut Terminal<B>, app: &'a mut App<'a>) -> io::Result<()> {
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

	let mut scrollCounter = 0;
	let scrollTimer = 100;
	let mut autoscroll = true;


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
    let mut needLoad = true;
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
                if app.past_events_list.items.len() > 10
                {
				    app.past_events_list.scroll();
                }
			}
			else
			{
				scrollCounter -= 1;
			}
		}

		// update terminal view
        terminal.draw(|f| ui(f, app));
		
        if needLoad
        {
            loaders::load_current_events(&mut app.curr_events.items).await;
            loaders::load_past_events(&mut app.past_events_list.items).await;
            loaders::load_leaderboard(&mut app.leaderboard_stats.items).await;
            loaders::load_writeups(&mut app.writeups.items).await;
            needLoad = false;
        }
    }

	inputThreadHandle.join().unwrap();
    Ok(())
}

/*
 * build_leaderboard
 * create leaderboard widget via App.leaderboard_stats 
 */
fn build_leaderboard<'a>(app:&'a mut App) -> Table<'a> {
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

fn build_current_events<'a>(app :&'a mut App, width: usize) -> List<'a> {
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
        .current_events_list
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

fn build_past_events<'a>(app :&'a mut App, width: usize) -> List<'a> {
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

fn build_writeups<'a>(app :&'a mut App, width: usize) -> List<'a> {
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

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
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

