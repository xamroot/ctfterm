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
mod crawler;

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T: Clone> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn scroll(&mut self) {
        self.items.push(self.items[0].clone());
        self.items.remove(0);
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
    current_events_list: StatefulList<&'a str>,
    past_events_list: StatefulList<(&'a str, &'a str)>,
    leaderboard_stats: StatefulList<(&'a str, &'a str, &'a str, &'a str)>,
    events: Vec<(&'a str, &'a str)>,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            current_events_list: StatefulList::with_items(vec![

            ]),
            past_events_list: StatefulList::with_items(vec![
            ]),
            leaderboard_stats: StatefulList::with_items(vec![
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
    // initial crawl of data
    let past_evts: Vec<Vec<String>> = crawler::get_past_events().await.unwrap();
    let evts: Vec<String> = crawler::crawl().await?;
    let leaderboard_stats: Vec<Vec<String>> = crawler::get_stats().await.unwrap();

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // initialize app
    let mut app = App::new();
    
    // update app with running events
    let mut i: usize = 0;
    for evt in &evts{
        if i > 0 {
            app.current_events_list.items.push(&evts[i]);
        }
        i += 1;
    }

    // update with past events of this year
    for evt in &past_evts {
        if &evt.len() > &0 {
            app.past_events_list.items.push((&evt[0], &evt[1]));
        }
    }
    
    // update app with leaderboard stats
    for stat in &leaderboard_stats {
        app.leaderboard_stats.items.push((&stat[0], &stat[1], &stat[3], &stat[2]));
    }

    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
	let mut running = true;
	let mut curr_keycode = 0;
    let mut isThread = false;
    let handle = std::thread::spawn(move || {
        while running == true
        {
		    if let Event::Key(key) = event::read().unwrap() {
				if let KeyCode::Char('q') = key.code {
                    running = false;
                    isThread = true;
                    break;
                }
            }
        }
    });


    if isThread == false {
        while running == true {

                terminal.draw(|f| ui(f, app));
                app.past_events_list.scroll();
        }
    }
    handle.join().unwrap();
    return Ok(());
}

/*
 * build_leaderboard
 * create leaderboard widget via App.leaderboard_stats 
 */
fn build_leaderboard<'a>(app:&'a mut App) -> Table<'a> {
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
        .border_style(Style::default().fg(Color::Red))
        .title_alignment(Alignment::Left);
    
    // set up headers
    let header_cells = [" ", " ", " ", " "]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(0);

    // create rows of leaderboard data
    let rows = app.leaderboard_stats.items.iter().map(|&(a,b,c,d)| {
        /*let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;*/
        let height = 1;
        //let cells = item.iter().map(|c| Cell::from(*c));
        let mut cells = vec![];
        cells.push( Cell::from( a ) );
        cells.push( Cell::from( b ) );
        cells.push( Cell::from( c ) );
        cells.push( Cell::from( d ) );
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
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
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
        .map(|&name| {
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
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
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
        .map(|&(name, data)| {
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
    let block = Block::default()
        .title(Span::styled(
            "Recent Write Ups",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(Color::Red))
        .borders(Borders::ALL);

    f.render_widget(block, bottom_chunks[0]);

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

