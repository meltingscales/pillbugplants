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
    PlantStem(u8),   // Main structural support, age 0-255
    PlantLeaf(u8),   // Photosynthesis organs, age 0-150
    PlantBud(u8),    // Growth points that become branches/flowers, age 0-50
    PlantFlower(u8), // Reproductive organs, age 0-100
    Pillbug(u8),     // Age of pillbug (0-255, dies when reaching 255)
    Nutrient,
}

impl TileType {
    fn to_char(self) -> char {
        match self {
            TileType::Empty => ' ',
            TileType::Dirt => '#',
            TileType::Sand => '.',
            TileType::Water => '~',
            TileType::PlantStem(_) => '|',
            TileType::PlantLeaf(_) => 'L',
            TileType::PlantBud(_) => 'o',
            TileType::PlantFlower(_) => '*',
            TileType::Pillbug(_) => 'B',
            TileType::Nutrient => '+',
        }
    }
    
    fn to_color(self) -> Color {
        match self {
            TileType::Empty => Color::Black,
            TileType::Dirt => Color::Rgb(101, 67, 33),
            TileType::Sand => Color::Yellow,
            TileType::Water => Color::Blue,
            TileType::PlantStem(age) => {
                let intensity = (255 - age as u16).max(80) as u8;
                Color::Rgb(intensity / 3, intensity, intensity / 4) // Brown-green stem
            },
            TileType::PlantLeaf(age) => {
                let intensity = (150 - age as u16).max(60) as u8;
                Color::Rgb(0, intensity, 0) // Green leaves
            },
            TileType::PlantBud(age) => {
                let intensity = (50 - age as u16).max(120) as u8;
                Color::Rgb(intensity, intensity / 2, 0) // Orange-ish buds
            },
            TileType::PlantFlower(age) => {
                let fade = age as u16;
                Color::Rgb((255 - fade).max(100) as u8, (200 - fade / 2).max(50) as u8, (255 - fade).max(100) as u8) // Pink-white flowers
            },
            TileType::Pillbug(age) => {
                let intensity = (255 - age as u16).max(50) as u8;
                Color::Rgb(intensity, intensity, intensity)
            },
            TileType::Nutrient => Color::Magenta,
        }
    }
    
    fn is_plant(self) -> bool {
        matches!(self, TileType::PlantStem(_) | TileType::PlantLeaf(_) | TileType::PlantBud(_) | TileType::PlantFlower(_))
    }
    
    fn is_plant_structural(self) -> bool {
        matches!(self, TileType::PlantStem(_))
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
        
        // Generate initial plant stems
        for _ in 0..(self.width / 20) {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(self.height - 12..self.height - 3);
            
            // Create a small initial plant with stem and maybe a leaf/bud
            if self.tiles[y][x] == TileType::Empty {
                self.tiles[y][x] = TileType::PlantStem(rng.gen_range(5..30));
                
                // 60% chance to add a leaf above stem
                if y > 0 && self.tiles[y - 1][x] == TileType::Empty && rng.gen_bool(0.6) {
                    self.tiles[y - 1][x] = TileType::PlantLeaf(rng.gen_range(0..20));
                }
                
                // 30% chance to add a bud somewhere nearby
                if rng.gen_bool(0.3) {
                    let directions = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0)];
                    if let Some(&(dx, dy)) = directions.choose(&mut rng) {
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;
                        if nx < self.width && ny < self.height && self.tiles[ny][nx] == TileType::Empty {
                            self.tiles[ny][nx] = TileType::PlantBud(0);
                        }
                    }
                }
            }
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
                    // Plant parts and pillbugs affected by gravity when not supported  
                    tile if tile.is_plant() => {
                        self.update_plant_physics(x, y, &mut new_tiles, tile);
                    }
                    TileType::Pillbug(_) => {
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
    
    fn update_plant_physics(&self, x: usize, y: usize, new_tiles: &mut Vec<Vec<TileType>>, tile: TileType) {
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
                        // Plant stems provide strong support, dirt/sand provide support
                        // Other plant parts provide weak support
                        if neighbor == TileType::Dirt || neighbor == TileType::Sand || 
                           neighbor.is_plant_structural() || 
                           (neighbor.is_plant() && !matches!(tile, TileType::PlantStem(_))) {
                            has_support = true;
                            break;
                        }
                    }
                }
                if has_support { break; }
            }
            
            // Fall if no support, but stems are more stable
            if !has_support {
                let below = self.tiles[y + 1][x];
                if below == TileType::Empty || below == TileType::Water {
                    new_tiles[y][x] = TileType::Empty;
                    new_tiles[y + 1][x] = tile;
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
                    // Plant stem - main structural component
                    TileType::PlantStem(age) => {
                        self.update_plant_stem(x, y, age, &mut new_tiles, &mut rng);
                    }
                    // Plant leaf - photosynthesis
                    TileType::PlantLeaf(age) => {
                        self.update_plant_leaf(x, y, age, &mut new_tiles, &mut rng);
                    }
                    // Plant bud - growth point
                    TileType::PlantBud(age) => {
                        self.update_plant_bud(x, y, age, &mut new_tiles, &mut rng);
                    }
                    // Plant flower - reproduction
                    TileType::PlantFlower(age) => {
                        self.update_plant_flower(x, y, age, &mut new_tiles, &mut rng);
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
                        
                        // Pillbugs eat plant parts for nutrients (prefer leaves and flowers)
                        let mut found_food = false;
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx < self.width && ny < self.height {
                                    let tile = self.tiles[ny][nx];
                                    if tile.is_plant() {
                                        let eat_chance = match tile {
                                            TileType::PlantLeaf(_) => 0.15,     // Prefer leaves
                                            TileType::PlantFlower(_) => 0.12,   // Like flowers
                                            TileType::PlantBud(_) => 0.08,      // Eat buds sometimes
                                            TileType::PlantStem(_) => 0.03,     // Rarely eat stems
                                            _ => 0.0,
                                        };
                                        if rng.gen_bool(eat_chance) {
                                            new_tiles[ny][nx] = TileType::Nutrient; // Plant part becomes nutrient
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
    
    fn update_plant_stem(&self, x: usize, y: usize, age: u8, new_tiles: &mut Vec<Vec<TileType>>, rng: &mut impl Rng) {
        let mut new_age = age.saturating_add(1);
        
        // Stems die after 255 ticks
        if new_age >= 255 {
            new_tiles[y][x] = TileType::Nutrient;
            return;
        }
        
        // Check for nutrients and consume them
        let mut found_nutrients = false;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                if nx < self.width && ny < self.height {
                    if self.tiles[ny][nx] == TileType::Nutrient {
                        new_tiles[ny][nx] = TileType::Empty;
                        new_age = new_age.saturating_sub(3);
                        found_nutrients = true;
                        break;
                    }
                }
            }
            if found_nutrients { break; }
        }
        
        new_tiles[y][x] = TileType::PlantStem(new_age);
        
        // Healthy stems can grow buds during the day
        if found_nutrients && self.is_day() && new_age < 150 && rng.gen_bool(0.03) {
            let directions = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0)];
            if let Some(&(dx, dy)) = directions.choose(rng) {
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                if nx < self.width && ny < self.height && new_tiles[ny][nx] == TileType::Empty {
                    new_tiles[ny][nx] = TileType::PlantBud(0);
                }
            }
        }
    }
    
    fn update_plant_leaf(&self, x: usize, y: usize, age: u8, new_tiles: &mut Vec<Vec<TileType>>, rng: &mut impl Rng) {
        let mut new_age = age.saturating_add(1);
        
        // Leaves die after 150 ticks
        if new_age >= 150 {
            new_tiles[y][x] = TileType::Nutrient;
            return;
        }
        
        // Leaves photosynthesize during day (slow aging)
        if self.is_day() {
            new_age = new_age.saturating_sub(1);
            
            // Healthy leaves can sometimes produce nutrients during the day
            if new_age < 100 && rng.gen_bool(0.02) {
                let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                if let Some(&(dx, dy)) = directions.choose(rng) {
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    if nx < self.width && ny < self.height && new_tiles[ny][nx] == TileType::Empty {
                        new_tiles[ny][nx] = TileType::Nutrient;
                    }
                }
            }
        }
        
        new_tiles[y][x] = TileType::PlantLeaf(new_age);
    }
    
    fn update_plant_bud(&self, x: usize, y: usize, age: u8, new_tiles: &mut Vec<Vec<TileType>>, rng: &mut impl Rng) {
        let new_age = age.saturating_add(1);
        
        // Buds develop into other structures
        if new_age >= 30 {
            // 50% chance to become a stem, 30% flower, 20% leaf
            let rand_val = rng.gen_range(0..10);
            if rand_val < 5 {
                new_tiles[y][x] = TileType::PlantStem(0);
            } else if rand_val < 8 {
                new_tiles[y][x] = TileType::PlantFlower(0);
            } else {
                new_tiles[y][x] = TileType::PlantLeaf(0);
            }
            return;
        }
        
        // Buds die if they can't find support from stems
        let mut has_stem_support = false;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                if nx < self.width && ny < self.height {
                    if let TileType::PlantStem(_) = self.tiles[ny][nx] {
                        has_stem_support = true;
                        break;
                    }
                }
            }
            if has_stem_support { break; }
        }
        
        if !has_stem_support {
            new_tiles[y][x] = TileType::Empty;
            return;
        }
        
        new_tiles[y][x] = TileType::PlantBud(new_age);
    }
    
    fn update_plant_flower(&self, x: usize, y: usize, age: u8, new_tiles: &mut Vec<Vec<TileType>>, rng: &mut impl Rng) {
        let new_age = age.saturating_add(1);
        
        // Flowers wither after 100 ticks
        if new_age >= 100 {
            new_tiles[y][x] = TileType::Nutrient;
            return;
        }
        
        // Flowers can spread seeds during the day
        if self.is_day() && new_age > 20 && new_age < 80 && rng.gen_bool(0.02) {
            let directions = [(-2, -1), (-1, -2), (0, -2), (1, -2), (2, -1), (2, 0), (1, 1), (0, 2), (-1, 1), (-2, 0)];
            if let Some(&(dx, dy)) = directions.choose(rng) {
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                if nx < self.width && ny < self.height && new_tiles[ny][nx] == TileType::Empty {
                    // Seeds need dirt/sand to grow
                    if ny + 1 < self.height && 
                       (self.tiles[ny + 1][nx] == TileType::Dirt || self.tiles[ny + 1][nx] == TileType::Sand) {
                        new_tiles[ny][nx] = TileType::PlantStem(0);
                    }
                }
            }
        }
        
        new_tiles[y][x] = TileType::PlantFlower(new_age);
    }
    
    fn is_day(&self) -> bool {
        self.day_cycle.sin() > 0.0
    }
}

struct App {
    world: World,
    show_taxonomy: bool,
}

impl App {
    fn new(width: usize, height: usize) -> Self {
        App {
            world: World::new(width, height),
            show_taxonomy: false,
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
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('t') => app.show_taxonomy = !app.show_taxonomy,
                    _ => {}
                }
            }
        }
        
        app.tick();
    }
}

fn ui(f: &mut Frame, app: &App) {
    let main_chunks = if app.show_taxonomy {
        Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Min(0), Constraint::Length(25)].as_ref())
            .split(f.area())
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Min(0)].as_ref())
            .split(f.area())
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(main_chunks[0]);

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
        "Tick: {} | {}{} | Press 'q' to quit | Press 't' for taxonomy",
        app.world.tick, day_night, rain_status
    ))
    .block(Block::default().title("Info").borders(Borders::ALL));
    f.render_widget(info, chunks[1]);

    // Render taxonomy panel if enabled
    if app.show_taxonomy {
        let taxonomy_text = vec![
            Line::from(vec![
                Span::styled(" ", Style::default().fg(Color::Black)),
                Span::raw(" = Empty space")
            ]),
            Line::from(vec![
                Span::styled("#", Style::default().fg(Color::Rgb(101, 67, 33))),
                Span::raw(" = Dirt (solid ground)")
            ]),
            Line::from(vec![
                Span::styled(".", Style::default().fg(Color::Yellow)),
                Span::raw(" = Sand (falls)")
            ]),
            Line::from(vec![
                Span::styled("~", Style::default().fg(Color::Blue)),
                Span::raw(" = Water (flows)")
            ]),
            Line::from(vec![
                Span::styled("|", Style::default().fg(Color::Rgb(80, 200, 60))),
                Span::raw(" = Plant Stem (structural)")
            ]),
            Line::from(vec![
                Span::styled("L", Style::default().fg(Color::Green)),
                Span::raw(" = Plant Leaf (photosynthesis)")
            ]),
            Line::from(vec![
                Span::styled("o", Style::default().fg(Color::Rgb(200, 100, 0))),
                Span::raw(" = Plant Bud (grows into parts)")
            ]),
            Line::from(vec![
                Span::styled("*", Style::default().fg(Color::Rgb(255, 150, 200))),
                Span::raw(" = Plant Flower (spreads seeds)")
            ]),
            Line::from("  - Stems: consume nutrients, grow buds"),
            Line::from("  - Leaves: photosynthesize, produce nutrients"),
            Line::from("  - Buds: develop into stems/leaves/flowers"),
            Line::from("  - Flowers: spread seeds during day"),
            Line::from(vec![
                Span::styled("B", Style::default().fg(Color::Gray)),
                Span::raw(" = Pillbug (ages 0-180)")
            ]),
            Line::from("  - Eats plant parts (prefers leaves)"),
            Line::from("  - Reproduces when fed"),
            Line::from("  - Gets darker with age"),
            Line::from(vec![
                Span::styled("+", Style::default().fg(Color::Magenta)),
                Span::raw(" = Nutrient (diffuses)")
            ]),
            Line::from("  - From decomposition"),
            Line::from("  - Consumed by plants"),
            Line::from(""),
            Line::from("Physics:"),
            Line::from("- Gravity affects all"),
            Line::from("- 8-way support check"),
            Line::from("- Rain spawns at night"),
            Line::from(""),
            Line::from("Ecosystem:"),
            Line::from("- Plants die → nutrients"),
            Line::from("- Bugs eat plants"),
            Line::from("- Closed nutrient loop"),
        ];

        let taxonomy_panel = Paragraph::new(taxonomy_text)
            .block(Block::default().title("Taxonomy").borders(Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(taxonomy_panel, main_chunks[1]);
    }
}