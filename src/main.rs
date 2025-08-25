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


#[derive(Clone, Copy, PartialEq)]
enum TileType {
    Empty,
    Dirt,
    Sand,
    Water,
    Plant(u8), // Age of plant (0-255, dies when reaching 255)
    Pillbug(u8), // Age of pillbug (0-255, dies when reaching 255)
    Nutrient,
}

impl TileType {
    fn to_char(self) -> char {
        match self {
            TileType::Empty => ' ',
            TileType::Dirt => '#',
            TileType::Sand => '.',
            TileType::Water => '~',
            TileType::Plant(_) => 'P',
            TileType::Pillbug(_) => 'B',
            TileType::Nutrient => '*',
        }
    }
    
    fn to_color(self) -> Color {
        match self {
            TileType::Empty => Color::Black,
            TileType::Dirt => Color::Rgb(101, 67, 33),
            TileType::Sand => Color::Yellow,
            TileType::Water => Color::Blue,
            TileType::Plant(age) => {
                let intensity = (255 - age as u16).max(50) as u8;
                Color::Rgb(0, intensity, 0)
            },
            TileType::Pillbug(age) => {
                let intensity = (255 - age as u16).max(50) as u8;
                Color::Rgb(intensity, intensity, intensity)
            },
            TileType::Nutrient => Color::Magenta,
        }
    }
}

struct World {
    tiles: Vec<Vec<TileType>>,
    width: usize,
    height: usize,
    tick: u64,
    day_cycle: f32,
    rain_intensity: f32,
}

impl World {
    fn new(width: usize, height: usize) -> Self {
        let tiles = vec![vec![TileType::Empty; width]; height];
        let mut world = World {
            tiles,
            width,
            height,
            tick: 0,
            day_cycle: 0.0,
            rain_intensity: 0.0,
        };
        
        world.generate_initial_world();
        world
    }
    
    fn generate_initial_world(&mut self) {
        let mut rng = rand::thread_rng();
        
        for y in self.height - 10..self.height {
            for x in 0..self.width {
                if rng.gen_bool(0.8) {
                    self.tiles[y][x] = TileType::Dirt;
                }
            }
        }
        
        for _ in 0..(self.width / 2) {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(self.height - 8..self.height);
            self.tiles[y][x] = TileType::Sand;
        }
        
        for _ in 0..(self.width / 4) {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(self.height - 5..self.height);
            self.tiles[y][x] = TileType::Water;
        }
        
        for _ in 0..(self.width / 16) {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(self.height - 15..self.height - 5);
            self.tiles[y][x] = TileType::Plant(rng.gen_range(10..50));
        }
        
        for _ in 0..(self.width / 30) {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(self.height - 10..self.height);
            self.tiles[y][x] = TileType::Pillbug(rng.gen_range(10..50));
        }
        
        // Add some initial nutrients
        for _ in 0..(self.width / 20) {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(self.height - 12..self.height);
            if self.tiles[y][x] == TileType::Empty {
                self.tiles[y][x] = TileType::Nutrient;
            }
        }
    }
    
    fn update(&mut self) {
        self.tick += 1;
        self.day_cycle = (self.tick as f32 * 0.01) % (2.0 * std::f32::consts::PI);
        
        // Rain cycle - more likely during night
        let mut rng = rand::thread_rng();
        if self.day_cycle.sin() < -0.3 && rng.gen_bool(0.05) {
            self.rain_intensity = rng.gen_range(0.1..0.8);
        } else if rng.gen_bool(0.02) {
            self.rain_intensity *= 0.95; // Rain gradually stops
        }
        
        self.spawn_rain();
        self.update_physics();
        self.diffuse_nutrients();
        self.update_life();
    }
    
    fn spawn_rain(&mut self) {
        if self.rain_intensity > 0.1 {
            let mut rng = rand::thread_rng();
            let drops_to_spawn = (self.rain_intensity * self.width as f32 * 0.1) as usize;
            
            for _ in 0..drops_to_spawn {
                let x = rng.gen_range(0..self.width);
                if self.tiles[0][x] == TileType::Empty {
                    self.tiles[0][x] = TileType::Water;
                }
            }
        }
    }
    
    fn update_physics(&mut self) {
        let mut new_tiles = self.tiles.clone();
        
        for y in (0..self.height - 1).rev() {
            for x in 0..self.width {
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
                                if nx < self.width && ny < self.height && self.tiles[ny][nx] == TileType::Empty {
                                    new_tiles[y][x] = TileType::Empty;
                                    new_tiles[ny][nx] = TileType::Water;
                                }
                            }
                        }
                    }
                    // Plants and pillbugs affected by gravity when not supported
                    TileType::Plant(_) | TileType::Pillbug(_) => {
                        if y + 1 < self.height {
                            // Check all 8 adjacent positions for support
                            let mut has_support = false;
                            for dy in -1..=1 {
                                for dx in -1..=1 {
                                    if dx == 0 && dy == 0 { continue; } // Skip self
                                    let nx = (x as i32 + dx) as usize;
                                    let ny = (y as i32 + dy) as usize;
                                    if nx < self.width && ny < self.height {
                                        let neighbor = self.tiles[ny][nx];
                                        // Any solid tile provides support (not empty, not water, not nutrients)
                                        if neighbor != TileType::Empty && neighbor != TileType::Water && neighbor != TileType::Nutrient {
                                            has_support = true;
                                            break;
                                        }
                                    }
                                }
                                if has_support { break; }
                            }
                            
                            // Fall if no support
                            if !has_support {
                                let below = self.tiles[y + 1][x];
                                if below == TileType::Empty || below == TileType::Water {
                                    new_tiles[y][x] = TileType::Empty;
                                    new_tiles[y + 1][x] = self.tiles[y][x];
                                    // If falling into water, water gets displaced
                                    if below == TileType::Water {
                                        // Try to move water to a nearby empty space
                                        let mut rng = rand::thread_rng();
                                        let directions = [(-1, 0), (1, 0), (0, -1)];
                                        if let Some(&(dx, dy)) = directions.choose(&mut rng) {
                                            let nx = (x as i32 + dx) as usize;
                                            let ny = ((y + 1) as i32 + dy) as usize;
                                            if nx < self.width && ny < self.height && self.tiles[ny][nx] == TileType::Empty {
                                                new_tiles[ny][nx] = TileType::Water;
                                            }
                                        }
                                    }
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
    
    fn diffuse_nutrients(&mut self) {
        let mut new_tiles = self.tiles.clone();
        
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                if self.tiles[y][x] == TileType::Nutrient {
                    let mut rng = rand::thread_rng();
                    if rng.gen_bool(0.1) {
                        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                        if let Some(&(dx, dy)) = directions.choose(&mut rng) {
                            let nx = (x as i32 + dx) as usize;
                            let ny = (y as i32 + dy) as usize;
                            if self.tiles[ny][nx] == TileType::Empty || self.tiles[ny][nx] == TileType::Water {
                                new_tiles[y][x] = TileType::Empty;
                                new_tiles[ny][nx] = TileType::Nutrient;
                            }
                        }
                    }
                }
            }
        }
        
        self.tiles = new_tiles;
    }
    
    fn update_life(&mut self) {
        let mut rng = rand::thread_rng();
        let mut new_tiles = self.tiles.clone();
        
        for y in 0..self.height {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    TileType::Plant(age) => {
                        let mut new_age = age.saturating_add(1);
                        let mut should_reproduce = false;
                        
                        // Plants age and may die
                        if new_age >= 200 {
                            // Plant dies and decomposes into nutrients
                            new_tiles[y][x] = TileType::Nutrient;
                            continue;
                        }
                        
                        // Plants need nutrients to survive and reproduce
                        let mut has_nutrients = false;
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx < self.width && ny < self.height {
                                    if let TileType::Nutrient = self.tiles[ny][nx] {
                                        has_nutrients = true;
                                        new_tiles[ny][nx] = TileType::Empty; // Consume nutrient
                                        new_age = new_age.saturating_sub(5); // Nutrients slow aging
                                        should_reproduce = self.day_cycle.sin() > 0.0 && rng.gen_bool(0.05);
                                        break;
                                    }
                                }
                            }
                            if has_nutrients { break; }
                        }
                        
                        // Without nutrients, age faster
                        if !has_nutrients {
                            new_age = new_age.saturating_add(1);
                        }
                        
                        new_tiles[y][x] = TileType::Plant(new_age);
                        
                        // Reproduction during day with nutrients
                        if should_reproduce {
                            let directions = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0)];
                            if let Some(&(dx, dy)) = directions.choose(&mut rng) {
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx < self.width && ny < self.height && new_tiles[ny][nx] == TileType::Empty {
                                    new_tiles[ny][nx] = TileType::Plant(0);
                                }
                            }
                        }
                    }
                    TileType::Pillbug(age) => {
                        let mut new_age = age.saturating_add(1);
                        let mut should_reproduce = false;
                        
                        // Pillbugs age and may die
                        if new_age >= 180 {
                            // Pillbug dies and decomposes into nutrients
                            new_tiles[y][x] = TileType::Nutrient;
                            continue;
                        }
                        
                        // Pillbugs eat plants for nutrients
                        let mut found_food = false;
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx < self.width && ny < self.height {
                                    if let TileType::Plant(_) = self.tiles[ny][nx] {
                                        if rng.gen_bool(0.1) { // 10% chance to eat plant
                                            new_tiles[ny][nx] = TileType::Nutrient; // Plant becomes nutrient
                                            new_age = new_age.saturating_sub(10); // Food slows aging significantly
                                            found_food = true;
                                            should_reproduce = rng.gen_bool(0.03);
                                            break;
                                        }
                                    }
                                }
                            }
                            if found_food { break; }
                        }
                        
                        // Without food, age faster (starve)
                        if !found_food {
                            new_age = new_age.saturating_add(2);
                        }
                        
                        // Movement
                        if rng.gen_bool(0.15) {
                            let directions = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
                            if let Some(&(dx, dy)) = directions.choose(&mut rng) {
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx < self.width && ny < self.height && new_tiles[ny][nx] == TileType::Empty {
                                    new_tiles[y][x] = TileType::Empty;
                                    new_tiles[ny][nx] = TileType::Pillbug(new_age);
                                    
                                    // Reproduction
                                    if should_reproduce && rng.gen_bool(0.5) {
                                        new_tiles[y][x] = TileType::Pillbug(0); // Baby pillbug
                                    }
                                } else {
                                    new_tiles[y][x] = TileType::Pillbug(new_age);
                                }
                            }
                        } else {
                            new_tiles[y][x] = TileType::Pillbug(new_age);
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
    fn new(width: usize, height: usize) -> Self {
        App {
            world: World::new(width, height),
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

    let size = terminal.size()?;
    let world_width = size.width.saturating_sub(4) as usize;
    let world_height = size.height.saturating_sub(6) as usize;
    
    let mut app = App::new(world_width, world_height);
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
    for y in 0..app.world.height {
        let mut spans = Vec::new();
        for x in 0..app.world.width {
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
    let rain_status = if app.world.rain_intensity > 0.1 {
        format!(" | Rain: {:.1}", app.world.rain_intensity)
    } else {
        String::new()
    };
    let info = Paragraph::new(format!(
        "Tick: {} | {}{} | Press 'q' to quit",
        app.world.tick, day_night, rain_status
    ))
    .block(Block::default().title("Info").borders(Borders::ALL));
    f.render_widget(info, chunks[1]);
}