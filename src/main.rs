mod stops;

use std::io::Write;
use std::thread;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle, style};

use crossterm::{
    ExecutableCommand,
    cursor::{self, MoveTo, MoveToColumn, MoveToNextLine},
    event, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use ordermap::OrderMap;

use std::cmp::Ordering;

use std::io;

use crate::stops::{Stop, get_stop, read_stops};

const UPDATE_FREQ: u64 = 500; // ms
const D_BETWEEN: usize = 6;

#[derive(Debug, Clone)]
struct Train {
    pub name: String,
    pub train_art: String,
    pub freq_update: u32,
    pub stops: OrderMap<String, Stop>,
}

#[derive(Debug, Clone)]
enum TrackState {
    Track1,
    Track2,
}

impl TrackState {
    pub fn art(&self) -> String {
        match self {
            Self::Track1 => "=/=/=/=/=/=/=/=/=/=/=/=/=/=/=/=/=/=/=/".to_string(),
            Self::Track2 => "/=/=/=/=/=/=/=/=/=/=/=/=/=/=/=/=/=/=/=".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct TrainInProgress {
    pub train: Train,
    pub pb: ProgressBar,
    pub sec_elapsed: f64,
    pub state: TrackState,
    pub stopped: bool,
    pub updated: u32,
}

impl TrainInProgress {
    pub fn new(train: Train, stops: &OrderMap<String, Stop>, start: &str, end: &str) -> Self {
        let time_distance = stops.get(end).unwrap().loc - stops.get(start).unwrap().loc;
        let style = ProgressStyle::with_template("{wide_bar}").unwrap();
        TrainInProgress {
            train,
            pb: ProgressBar::new(time_distance as u64).with_style(style),
            state: TrackState::Track1,
            stopped: false,
            sec_elapsed: 0.0,
            updated: 0,
        }
    }

    pub fn change_track(&mut self) {
        if !self.stopped && (self.updated % self.train.freq_update) == 0 {
            self.state = match self.state {
                TrackState::Track1 => TrackState::Track2,
                TrackState::Track2 => TrackState::Track1,
            };
        }
        self.updated += 1;
    }

    pub fn station_verb(&self) -> &str {
        if self.stopped { "Current" } else { "Next" }
    }
}

#[derive(Debug, Clone)]
struct Route {
    start: String,
    end: String,
}

fn format_time(secs: f64) -> String {
    if secs < 60.0 {
        format!("{:.0} seconds", secs)
    } else if secs < 3600.0 {
        format!("{:.0} minutes", secs / 60.0)
    } else if secs < 3600.0 * 24.0 {
        format!("{:.1} hours", secs / 3600.0)
    } else {
        format!("{:.1} days", secs / (3600.0 * 24.0))
    }
}

impl Route {
    fn time_distance(&self, stops: &OrderMap<String, Stop>) -> f64 {
        stops.get(&self.end).unwrap().loc - stops.get(&self.start).unwrap().loc 
    }

    fn render(self, trains: &[Train]) {
        // TODO: Get rid of miles progress:W
        let mut trains_going: Vec<_> = trains
            .iter()
            .map(|train| TrainInProgress::new(train.clone(), &train.stops, &self.start, &self.end))
            .collect();


        while trains_going.iter().any(|tg| tg.sec_elapsed < self.time_distance(&tg.train.stops)) {
            for (i, tg) in trains_going.iter_mut().enumerate() {
                let start_sec = tg.train.stops.get(&self.start).unwrap().loc;
                let (columns, _) = terminal::size().unwrap();
                if tg.sec_elapsed >= self.time_distance(&tg.train.stops) {
                    break;
                }
                // TODO: Fixes needed: secs_remaining, variable speed, IO, devpost.
                let secs_remaining = self.time_distance(&tg.train.stops) - tg.sec_elapsed;
                let (prev_station, next_station) = get_stop(&tg.train.stops, tg.sec_elapsed + start_sec);
                let time_str = format_time(secs_remaining);
                execute!(
                    io::stdout(),
                    MoveTo(0, (D_BETWEEN * i) as u16),
                    Clear(ClearType::UntilNewLine),
                    Print(tg.train.name.clone()),
                    MoveToColumn(columns - (time_str.len() as u16)),
                    Print(time_str),
                    MoveTo(0, (D_BETWEEN * i) as u16 + 2),
                    Print(&tg.train.train_art),
                    MoveToNextLine(1),
                    Print(&tg.state.art()),
                    MoveToNextLine(1),
                    // could be long name
                    Print(format!(
                        "{} Station: {}",
                        tg.station_verb(),
                        next_station.short_name
                    )),
                )
                .unwrap();
                tg.change_track();
                io::stdout()
                    .execute(MoveTo(0, (D_BETWEEN * i) as u16 + 1))
                    .unwrap();
                tg.pb.set_position(tg.sec_elapsed as u64);
                // TODO: Have faster time mode?
                tg.sec_elapsed += (UPDATE_FREQ as f64) / 1000.0;
            }
            // NOTE: Change from 1 for speedup.
            thread::sleep(Duration::from_millis(UPDATE_FREQ) / 1);
        }
    }
}

fn print_stations(stops: &OrderMap<String, Stop>) {
   for (k, _) in stops {
        println!("{}", k);
   } 
}

fn print_welcome() {
    println!(
        "Welcome to TRAIN SIMULATOR!\nEnter starting station or type \"ls\" for list of available stations: "
    );
    io::stdout().flush().unwrap();
}

fn print_welcome2() {
    println!(
        "Welcome to TRAIN SIMULATOR!\nEnter ending station or type \"ls\" for list of available stations: "
    );
    io::stdout().flush().unwrap();
}

fn main() {
    let (slow_stops, fast_stops) = read_stops();
    print_welcome();
    let mut input = String::new();

    input.clear();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input = input.trim().to_string();

    let mut src: Option<String> = None;
    let mut dest: Option<String> = None;

    while input != "exit" {
        if input == "ls" {
            print_stations(&slow_stops);
            print_welcome();
            input.clear();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            input = input.trim().to_string();
        } else {
            if slow_stops.contains_key(&input) {
                src = Some(input.clone());
                break;
            } else {
                println!("Invalid statement; try again");
                print_welcome();
                input.clear();
                io::stdin().read_line(&mut input).expect("Failed to read line");
                input = input.trim().to_string();
            }
        }
    }

    if input == "exit" {
        return;
    }

    print_welcome2();
    input.clear();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input = input.trim().to_string();

    while input.trim() != "exit" {
        if input.trim() == "ls" {
            print_stations(&slow_stops);
            print_welcome2();
            input.clear();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            input = input.trim().to_string();
        } else {
            if slow_stops.contains_key(&input) {
                dest = Some(input.clone());
                break;
            } else {
                println!("Invalid statement; try again");
                print_welcome2();
                input.clear();
                io::stdin().read_line(&mut input).expect("Failed to read line");
                input = input.trim().to_string();
            }
        }
    }

    if input == "exit" {
        return;
    }

    io::stdout().execute(EnterAlternateScreen).unwrap();

    let train1 = Train {
        name: "Amtrak Pacific Surfliner".to_string(),
        train_art: "[_□□□□_][_□□□□_][_□□□□_\\".to_string(),
        freq_update: 3,
        stops: slow_stops.clone(),
    };
    let train2 = Train {
        name: "CA High Speed Rail".to_string(),
        train_art: "________|_______|_______\\".to_string(),
        freq_update: 1,
        stops: fast_stops.clone(),
    };

    let route = Route {
        start: src.unwrap().clone(),
        end: dest.unwrap().clone(),
    };
    route.render(&[train1.clone(), train2.clone()]);
    io::stdout().execute(LeaveAlternateScreen).unwrap();
}

// TODO: speed algorithm, stop moving

