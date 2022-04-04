use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span,Spans},
    widgets::{List, ListItem, ListState, Block, BorderType, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

pub struct StatefulList<T> {
    state: ListState,
    pub items: Vec<T>,
    idx: usize
}

impl<T: Clone> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
            idx: 0,
        }
    }

    pub fn update(&mut self, items: &Vec<T>)
    {
        let mut i = 0;
        while i < items.len()
        {
            self.items.push(items[i].clone());
            i+=1;
        }
    }

    pub fn scroll(&mut self) {
        self.items.push(self.items[0].clone());
        self.items.remove(0);
    }

    pub fn move_down(&mut self){
        if self.idx < self.items.len()-1
        {
            self.items.push(self.items[0].clone());
            self.items.remove(0);
            self.idx += 1;
        }
    }

    pub fn get(&mut self, index: usize) -> &T
    {
        return &self.items[index]; 
    }

    pub fn set(&mut self, data: T, index: usize)
    {
        self.items[index] = data;
    }

    pub fn move_up(&mut self){
        if self.idx > 0
        {
            self.items.insert(0, self.items[self.items.len()-1].clone());
            self.items.remove(self.items.len()-1);
            self.idx -= 1;
        }
    }

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

pub struct App<'a> {
    pub focused: i16,
    pub term_height: u16,
    pub curr_events: StatefulList<String>,
    pub current_events_list: StatefulList<String>,
    pub past_events_list: StatefulList<(String, String)>,
    pub leaderboard_stats: StatefulList<(String, String, String, String)>,
    pub writeups: StatefulList<(String, String, String, String, String)>,
    pub events: Vec<(&'a str, &'a str)>,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
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
    pub fn on_tick(&mut self) {
        let event = self.events.remove(0);
        self.events.push(event);
    }
}
