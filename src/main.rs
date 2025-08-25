use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use rand::{Rng, seq::SliceRandom};
use std::io;

const WORLD_WIDTH: usize = 80;
const WORLD_HEIGHT: usize = 40;

#[derive(Clone, Copy, PartialEq)]
enum TileType {
    Empty,
    Dirt,
    Sand,
    Water,
    Plant,
    Pillbug,
    Nutrient,
}

impl TileType {
    fn to_char(self) -> char {
        match self {
            TileType::Empty => ' ',
            TileType::Dirt => '#',
            TileType::Sand => '.',
            TileType::Water => '~',
            TileType::Plant => 'P',
            TileType::Pillbug => 'B',
            TileType::Nutrient => '*',
        }
    }
    
    fn to_color(self) -> Color {
        match self {
            TileType::Empty => Color::Black,
            TileType::Dirt => Color::Rgb(101, 67, 33),
            TileType::Sand => Color::Yellow,
            TileType::Water => Color::Blue,
            TileType::Plant => Color::Green,
            TileType::Pillbug => Color::Gray,
            TileType::Nutrient => Color::Magenta,
        }
    }
}

struct World {
    tiles: [[TileType; WORLD_WIDTH]; WORLD_HEIGHT],
    tick: u64,
    day_cycle: f32,
}

impl World {
    fn new() -> Self {
        let mut world = World {
            tiles: [[TileType::Empty; WORLD_WIDTH]; WORLD_HEIGHT],
            tick: 0,
            day_cycle: 0.0,
        };
        
        world.generate_initial_world();
        world
    }
    
    fn generate_initial_world(&mut self) {
        let mut rng = rand::thread_rng();
        
        for y in WORLD_HEIGHT - 10..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                if rng.gen_bool(0.8) {
                    self.tiles[y][x] = TileType::Dirt;
                }
            }
        }
        
        for _ in 0..50 {
            let x = rng.gen_range(0..WORLD_WIDTH);
            let y = rng.gen_range(WORLD_HEIGHT - 8..WORLD_HEIGHT);
            self.tiles[y][x] = TileType::Sand;
        }
        
        for _ in 0..20 {
            let x = rng.gen_range(0..WORLD_WIDTH);
            let y = rng.gen_range(WORLD_HEIGHT - 5..WORLD_HEIGHT);
            self.tiles[y][x] = TileType::Water;
        }
        
        for _ in 0..5 {
            let x = rng.gen_range(0..WORLD_WIDTH);
            let y = rng.gen_range(WORLD_HEIGHT - 15..WORLD_HEIGHT - 5);
            self.tiles[y][x] = TileType::Plant;
        }
        
        for _ in 0..3 {
            let x = rng.gen_range(0..WORLD_WIDTH);
            let y = rng.gen_range(WORLD_HEIGHT - 10..WORLD_HEIGHT);
            self.tiles[y][x] = TileType::Pillbug;
        }
    }
    
    fn update(&mut self) {
        self.tick += 1;
        self.day_cycle = (self.tick as f32 * 0.01) % (2.0 * std::f32::consts::PI);
        
        self.update_physics();
        self.update_life();
    }
    
    fn update_physics(&mut self) {
        let mut new_tiles = self.tiles;
        
        for y in (0..WORLD_HEIGHT - 1).rev() {
            for x in 0..WORLD_WIDTH {
                match self.tiles[y][x] {
                    TileType::Sand | TileType::Water => {
                        if self.tiles[y + 1][x] == TileType::Empty {
                            new_tiles[y][x] = TileType::Empty;
                            new_tiles[y + 1][x] = self.tiles[y][x];
                        } else if self.tiles[y][x] == TileType::Water {
                            let mut rng = rand::thread_rng();
                            let directions = [(-1, 0), (1, 0)];
                            if let Some(&(dx, dy)) = directions.choose(&mut rng) {
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx < WORLD_WIDTH && ny < WORLD_HEIGHT && self.tiles[ny][nx] == TileType::Empty {
                                    new_tiles[y][x] = TileType::Empty;
                                    new_tiles[ny][nx] = TileType::Water;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        
        self.tiles = new_tiles;
    }
    
    fn update_life(&mut self) {
        let mut rng = rand::thread_rng();
        let mut new_tiles = self.tiles;
        
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                match self.tiles[y][x] {
                    TileType::Plant => {
                        if self.day_cycle.sin() > 0.0 && rng.gen_bool(0.02) {
                            let directions = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0)];
                            if let Some(&(dx, dy)) = directions.choose(&mut rng) {
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx < WORLD_WIDTH && ny < WORLD_HEIGHT && self.tiles[ny][nx] == TileType::Empty {
                                    new_tiles[ny][nx] = TileType::Plant;
                                }
                            }
                        }
                    }
                    TileType::Pillbug => {
                        if rng.gen_bool(0.1) {
                            let directions = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
                            if let Some(&(dx, dy)) = directions.choose(&mut rng) {
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx < WORLD_WIDTH && ny < WORLD_HEIGHT && self.tiles[ny][nx] == TileType::Empty {
                                    new_tiles[y][x] = TileType::Empty;
                                    new_tiles[ny][nx] = TileType::Pillbug;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        
        self.tiles = new_tiles;
    }
    
    fn is_day(&self) -> bool {
        self.day_cycle.sin() > 0.0
    }
}

struct App {
    world: World,
}

impl App {
    fn new() -> Self {
        App {
            world: World::new(),
        }
    }
    
    fn tick(&mut self) {
        self.world.update();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
        
        app.tick();
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(f.area());

    let mut lines = Vec::new();
    for y in 0..WORLD_HEIGHT {
        let mut spans = Vec::new();
        for x in 0..WORLD_WIDTH {
            let tile = app.world.tiles[y][x];
            spans.push(Span::styled(
                tile.to_char().to_string(),
                Style::default().fg(tile.to_color()),
            ));
        }
        lines.push(Line::from(spans));
    }

    let world_block = Paragraph::new(lines)
        .block(Block::default().title("Pillbug Plants").borders(Borders::ALL));
    f.render_widget(world_block, chunks[0]);

    let day_night = if app.world.is_day() { "Day" } else { "Night" };
    let info = Paragraph::new(format!(
        "Tick: {} | {} | Press 'q' to quit",
        app.world.tick, day_night
    ))
    .block(Block::default().title("Info").borders(Borders::ALL));
    f.render_widget(info, chunks[1]);
}