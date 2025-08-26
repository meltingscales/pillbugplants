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
use std::env;
use std::fs::File;
use std::io::Write;
use std::fmt;


#[derive(Clone, Copy, PartialEq)]
enum Size {
    Small = 0,   // Faster growth, shorter life, weaker
    Medium = 1,  // Normal values  
    Large = 2,   // Slower growth, longer life, stronger
}

impl Size {
    fn lifespan_multiplier(self) -> f32 {
        match self {
            Size::Small => 0.7,   // 30% shorter life
            Size::Medium => 1.0,  // Normal lifespan
            Size::Large => 1.4,   // 40% longer life
        }
    }
    
    fn growth_rate_multiplier(self) -> f32 {
        match self {
            Size::Small => 1.3,   // 30% faster growth/reproduction
            Size::Medium => 1.0,  // Normal rate
            Size::Large => 0.8,   // 20% slower growth/reproduction
        }
    }
    
    fn to_char_modifier(self, base_char: char) -> char {
        match (self, base_char) {
            (Size::Small, '|') => 'i',    // Small stem
            (Size::Small, 'L') => 'l',    // Small leaf
            (Size::Small, 'o') => '°',    // Small bud
            (Size::Small, '*') => '·',    // Small flower
            (Size::Small, '@') => 'ó',    // Small head
            (Size::Small, 'O') => 'o',    // Small body
            (Size::Small, 'w') => 'v',    // Small legs
            (Size::Large, '|') => '║',    // Large stem
            (Size::Large, 'L') => 'Ł',    // Large leaf
            (Size::Large, 'o') => 'O',    // Large bud
            (Size::Large, '*') => '✱',    // Large flower
            (Size::Large, '@') => '●',    // Large head
            (Size::Large, 'O') => '●',    // Large body
            (Size::Large, 'w') => 'W',    // Large legs
            _ => base_char, // Medium size keeps original char
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum TileType {
    Empty,
    Dirt,
    Sand,
    Water,
    PlantStem(u8, Size),   // Main structural support, age 0-255, size
    PlantLeaf(u8, Size),   // Photosynthesis organs, age 0-150, size
    PlantBud(u8, Size),    // Growth points that become branches/flowers, age 0-50, size
    PlantFlower(u8, Size), // Reproductive organs, age 0-100, size
    PlantWithered(u8, Size), // Dying plant part, age 0-30 before becoming nutrient, size
    PillbugHead(u8, Size),    // Head segment of pillbug, age 0-180, size
    PillbugBody(u8, Size),    // Body segment of pillbug, age 0-180, size
    PillbugLegs(u8, Size),    // Leg segment of pillbug, age 0-180, size
    PillbugDecaying(u8, Size), // Dying pillbug part, age 0-20 before becoming nutrient, size
    Nutrient,
}

impl TileType {
    fn to_char(self) -> char {
        match self {
            TileType::Empty => ' ',
            TileType::Dirt => '#',
            TileType::Sand => '.',
            TileType::Water => '~',
            TileType::PlantStem(_, size) => size.to_char_modifier('|'),
            TileType::PlantLeaf(_, size) => size.to_char_modifier('L'),
            TileType::PlantBud(_, size) => size.to_char_modifier('o'),
            TileType::PlantFlower(_, size) => size.to_char_modifier('*'),
            TileType::PlantWithered(_, size) => size.to_char_modifier('x'), // Withered plants
            TileType::PillbugHead(_, size) => size.to_char_modifier('@'),
            TileType::PillbugBody(_, size) => size.to_char_modifier('O'),
            TileType::PillbugLegs(_, size) => size.to_char_modifier('w'),
            TileType::PillbugDecaying(_, size) => size.to_char_modifier('░'), // Decaying pillbugs
            TileType::Nutrient => '+',
        }
    }
    
    fn to_color(self) -> Color {
        match self {
            TileType::Empty => Color::Black,
            TileType::Dirt => Color::Rgb(101, 67, 33),
            TileType::Sand => Color::Yellow,
            TileType::Water => Color::Blue,
            TileType::PlantStem(age, size) => {
                let base_intensity = (255 - age as u16).max(80) as u8;
                let size_boost = match size {
                    Size::Small => 0.85,   // Slightly dimmer
                    Size::Medium => 1.0,   // Normal
                    Size::Large => 1.15,   // Slightly brighter
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity / 3, intensity, intensity / 4) // Brown-green stem
            },
            TileType::PlantLeaf(age, size) => {
                let base_intensity = (150 - age as u16).max(60) as u8;
                let size_boost = match size {
                    Size::Small => 0.85,
                    Size::Medium => 1.0,
                    Size::Large => 1.15,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(0, intensity, 0) // Green leaves
            },
            TileType::PlantBud(age, size) => {
                let base_intensity = (50 - age as u16).max(120) as u8;
                let size_boost = match size {
                    Size::Small => 0.85,
                    Size::Medium => 1.0,
                    Size::Large => 1.15,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity, intensity / 2, 0) // Orange-ish buds
            },
            TileType::PlantFlower(age, size) => {
                let fade = age as u16;
                let base_red = (255 - fade).max(100) as u8;
                let base_green = (200 - fade / 2).max(50) as u8;
                let base_blue = (255 - fade).max(100) as u8;
                let size_boost = match size {
                    Size::Small => 0.85,
                    Size::Medium => 1.0,
                    Size::Large => 1.15,
                };
                let red = (base_red as f32 * size_boost).min(255.0) as u8;
                let green = (base_green as f32 * size_boost).min(255.0) as u8;
                let blue = (base_blue as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(red, green, blue) // Pink-white flowers
            },
            TileType::PlantWithered(age, size) => {
                let decay_progress = age as f32 / 30.0; // 0.0 = fresh withered, 1.0 = almost nutrient
                let base_intensity = (100.0 * (1.0 - decay_progress * 0.6)) as u8; // Darken over time
                let size_boost = match size {
                    Size::Small => 0.8,
                    Size::Medium => 1.0,
                    Size::Large => 1.2,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity, intensity / 2, 0) // Brown withered color
            },
            TileType::PillbugHead(age, size) => {
                let base_intensity = (180 - age as u16).max(60) as u8;
                let size_boost = match size {
                    Size::Small => 0.8,
                    Size::Medium => 1.0,
                    Size::Large => 1.2,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity.saturating_add(20), intensity, intensity.saturating_sub(10)) // Slightly reddish head
            },
            TileType::PillbugBody(age, size) => {
                let base_intensity = (180 - age as u16).max(50) as u8;
                let size_boost = match size {
                    Size::Small => 0.8,
                    Size::Medium => 1.0,
                    Size::Large => 1.2,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity, intensity, intensity) // Gray body
            },
            TileType::PillbugLegs(age, size) => {
                let base_intensity = (180 - age as u16).max(40) as u8;
                let size_boost = match size {
                    Size::Small => 0.8,
                    Size::Medium => 1.0,
                    Size::Large => 1.2,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity.saturating_sub(20), intensity.saturating_sub(10), intensity) // Slightly bluish legs
            },
            TileType::PillbugDecaying(age, size) => {
                let decay_progress = age as f32 / 20.0; // 0.0 = fresh decay, 1.0 = almost nutrient
                let base_intensity = (80.0 * (1.0 - decay_progress * 0.7)) as u8; // Darken significantly over time
                let size_boost = match size {
                    Size::Small => 0.7,
                    Size::Medium => 1.0,
                    Size::Large => 1.3,
                };
                let intensity = (base_intensity as f32 * size_boost).min(255.0) as u8;
                Color::Rgb(intensity, intensity / 3, intensity / 2) // Dark brownish-red decay color
            },
            TileType::Nutrient => Color::Magenta,
        }
    }
    
    fn is_plant(self) -> bool {
        matches!(self, TileType::PlantStem(_, _) | TileType::PlantLeaf(_, _) | TileType::PlantBud(_, _) | TileType::PlantFlower(_, _) | TileType::PlantWithered(_, _))
    }
    
    
    fn is_pillbug(self) -> bool {
        matches!(self, TileType::PillbugHead(_, _) | TileType::PillbugBody(_, _) | TileType::PillbugLegs(_, _) | TileType::PillbugDecaying(_, _))
    }
    
    fn get_size(self) -> Option<Size> {
        match self {
            TileType::PlantStem(_, size) | TileType::PlantLeaf(_, size) | 
            TileType::PlantBud(_, size) | TileType::PlantFlower(_, size) | TileType::PlantWithered(_, size) |
            TileType::PillbugHead(_, size) | TileType::PillbugBody(_, size) | TileType::PillbugLegs(_, size) | TileType::PillbugDecaying(_, size) => Some(size),
            _ => None,
        }
    }
    
}

fn random_size(rng: &mut impl Rng) -> Size {
    match rng.gen_range(0..10) {
        0..=2 => Size::Small,   // 30% small
        3..=6 => Size::Medium,  // 40% medium  
        7..=9 => Size::Large,   // 30% large
        _ => Size::Medium,
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
                let plant_size = random_size(&mut rng);
                self.tiles[y][x] = TileType::PlantStem(rng.gen_range(5..30), plant_size);
                
                // 60% chance to add a leaf above stem
                if y > 0 && self.tiles[y - 1][x] == TileType::Empty && rng.gen_bool(0.6) {
                    self.tiles[y - 1][x] = TileType::PlantLeaf(rng.gen_range(0..20), plant_size);
                }
                
                // 30% chance to add a bud somewhere nearby
                if rng.gen_bool(0.3) {
                    let directions = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0)];
                    if let Some(&(dx, dy)) = directions.choose(&mut rng) {
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;
                        if nx < self.width && ny < self.height && self.tiles[ny][nx] == TileType::Empty {
                            self.tiles[ny][nx] = TileType::PlantBud(0, plant_size);
                        }
                    }
                }
            }
        }
        
        // Generate initial multi-segment pillbugs
        for _ in 0..(self.width / 30) {
            let x = rng.gen_range(1..self.width - 1);
            let y = rng.gen_range(self.height - 10..self.height);
            let age = rng.gen_range(10..50);
            let pillbug_size = random_size(&mut rng);
            
            // Create a 3-segment pillbug: head, body, legs
            if self.tiles[y][x] == TileType::Empty {
                self.tiles[y][x] = TileType::PillbugHead(age, pillbug_size);
                
                // Try to place body behind head
                let directions = [(-1, 0), (1, 0), (0, 1), (0, -1)];
                if let Some(&(dx, dy)) = directions.choose(&mut rng) {
                    let body_x = (x as i32 + dx) as usize;
                    let body_y = (y as i32 + dy) as usize;
                    if body_x < self.width && body_y < self.height && self.tiles[body_y][body_x] == TileType::Empty {
                        self.tiles[body_y][body_x] = TileType::PillbugBody(age, pillbug_size);
                        
                        // Try to place legs adjacent to body
                        let leg_directions = [(-1, 0), (1, 0), (0, 1), (0, -1)];
                        if let Some(&(ldx, ldy)) = leg_directions.choose(&mut rng) {
                            let legs_x = (body_x as i32 + ldx) as usize;
                            let legs_y = (body_y as i32 + ldy) as usize;
                            if legs_x < self.width && legs_y < self.height && self.tiles[legs_y][legs_x] == TileType::Empty {
                                self.tiles[legs_y][legs_x] = TileType::PillbugLegs(age, pillbug_size);
                            }
                        }
                    }
                }
            }
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
        self.spawn_entities();
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
                    // Pillbug segments affected by gravity when not supported
                    tile if tile.is_pillbug() => {
                        self.update_pillbug_physics(x, y, &mut new_tiles, tile);
                    }
                    _ => {}
                }
            }
        }
        
        self.tiles = new_tiles;
    }
    
    fn update_plant_physics(&self, x: usize, y: usize, new_tiles: &mut [Vec<TileType>], tile: TileType) {
        if y + 1 < self.height {
            // Different plant types have different support requirements
            let (needs_strong_support, stability_threshold) = match tile {
                TileType::PlantStem(_, size) => {
                    // Stems need solid ground or other stems, stability varies by size
                    let threshold = match size {
                        Size::Large => 0.9,   // Large stems are more stable
                        Size::Medium => 0.7,  // Medium stability
                        Size::Small => 0.5,   // Small stems less stable
                    };
                    (true, threshold)
                },
                TileType::PlantLeaf(_, size) => {
                    // Leaves need any plant connection, less stability needed
                    let threshold = match size {
                        Size::Large => 0.8,   // Large leaves more stable
                        Size::Medium => 0.6,  
                        Size::Small => 0.4,   // Small leaves fall easily
                    };
                    (false, threshold)
                },
                TileType::PlantBud(_, _) => (false, 0.3),  // Buds are fragile
                TileType::PlantFlower(_, _) => (false, 0.4), // Flowers need some support
                TileType::PlantWithered(_, _) => (false, 0.1), // Withered plants very unstable
                _ => (false, 0.5),
            };
            
            // Count support strength from adjacent tiles
            let mut support_strength = 0.0;
            let mut support_count = 0;
            
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 { continue; } // Skip self
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    if nx < self.width && ny < self.height {
                        let neighbor = self.tiles[ny][nx];
                        let support_value = match neighbor {
                            TileType::Dirt | TileType::Sand => 1.0,  // Solid ground = full support
                            TileType::PlantStem(_, neighbor_size) => {
                                // Stem support strength varies by size
                                match neighbor_size {
                                    Size::Large => 0.9,
                                    Size::Medium => 0.7,
                                    Size::Small => 0.5,
                                }
                            },
                            TileType::PlantLeaf(_, _) => 0.3,  // Leaves provide weak support
                            TileType::PlantBud(_, _) => 0.1,   // Buds provide minimal support
                            TileType::PlantFlower(_, _) => 0.2, // Flowers provide some support
                            TileType::PlantWithered(_, _) => 0.05, // Withered plants very weak support
                            _ => 0.0,
                        };
                        
                        if support_value > 0.0 {
                            support_strength += support_value;
                            support_count += 1;
                        }
                    }
                }
            }
            
            // Calculate average support strength
            let avg_support = if support_count > 0 { 
                support_strength / support_count as f32 
            } else { 
                0.0 
            };
            
            // Check if support is sufficient
            let has_support = if needs_strong_support {
                // Stems need either solid ground or strong plant support
                support_strength >= 1.0 || avg_support >= stability_threshold
            } else {
                // Other plants just need sufficient average support
                avg_support >= stability_threshold
            };
            
            // Fall if insufficient support
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
    
    fn update_pillbug_physics(&self, x: usize, y: usize, new_tiles: &mut [Vec<TileType>], tile: TileType) {
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
                    TileType::PlantStem(age, size) => {
                        self.update_plant_stem(x, y, age, size, &mut new_tiles, &mut rng);
                    }
                    // Plant leaf - photosynthesis
                    TileType::PlantLeaf(age, size) => {
                        self.update_plant_leaf(x, y, age, size, &mut new_tiles, &mut rng);
                    }
                    // Plant bud - growth point
                    TileType::PlantBud(age, size) => {
                        self.update_plant_bud(x, y, age, size, &mut new_tiles, &mut rng);
                    }
                    // Plant flower - reproduction
                    TileType::PlantFlower(age, size) => {
                        self.update_plant_flower(x, y, age, size, &mut new_tiles, &mut rng);
                    }
                    // Plant withered - gradual decay
                    TileType::PlantWithered(age, size) => {
                        self.update_plant_withered(x, y, age, size, &mut new_tiles, &mut rng);
                    }
                    // Handle pillbug segments
                    TileType::PillbugHead(age, size) => {
                        self.update_pillbug_head(x, y, age, size, &mut new_tiles, &mut rng);
                    }
                    TileType::PillbugBody(age, size) => {
                        self.update_pillbug_body(x, y, age, size, &mut new_tiles, &mut rng);
                    }
                    TileType::PillbugLegs(age, size) => {
                        self.update_pillbug_legs(x, y, age, size, &mut new_tiles, &mut rng);
                    }
                    TileType::PillbugDecaying(age, size) => {
                        self.update_pillbug_decaying(x, y, age, size, &mut new_tiles, &mut rng);
                    }
                    _ => {}
                }
            }
        }
        
        self.tiles = new_tiles;
    }
    
    fn update_plant_stem(&self, x: usize, y: usize, age: u8, size: Size, new_tiles: &mut [Vec<TileType>], rng: &mut impl Rng) {
        let aging_rate = (1.0 / size.lifespan_multiplier()) as u8;
        let mut new_age = age.saturating_add(aging_rate);
        
        // Stems die after adjusted lifespan - transition to withered state
        let max_age = (255.0 * size.lifespan_multiplier()) as u8;
        if new_age >= max_age {
            new_tiles[y][x] = TileType::PlantWithered(0, size);
            return;
        }
        
        // Check for nutrients and consume them
        let mut found_nutrients = false;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                if nx < self.width && ny < self.height
                    && self.tiles[ny][nx] == TileType::Nutrient {
                    new_tiles[ny][nx] = TileType::Empty;
                    new_age = new_age.saturating_sub(3);
                    found_nutrients = true;
                    break;
                }
            }
            if found_nutrients { break; }
        }
        
        new_tiles[y][x] = TileType::PlantStem(new_age, size);
        
        // Healthy stems can grow buds during the day - rate affected by size
        let bud_chance = 0.03 * size.growth_rate_multiplier();
        if found_nutrients && self.is_day() && new_age < (max_age as u16 * 2 / 3) as u8 && rng.gen_bool(bud_chance as f64) {
            let directions = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0)];
            if let Some(&(dx, dy)) = directions.choose(rng) {
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                if nx < self.width && ny < self.height && new_tiles[ny][nx] == TileType::Empty {
                    // New buds inherit parent size with slight variation
                    let bud_size = if rng.gen_bool(0.8) { 
                        size 
                    } else { 
                        random_size(rng) 
                    };
                    new_tiles[ny][nx] = TileType::PlantBud(0, bud_size);
                }
            }
        }
    }
    
    fn update_plant_leaf(&self, x: usize, y: usize, age: u8, size: Size, new_tiles: &mut [Vec<TileType>], rng: &mut impl Rng) {
        let aging_rate = (1.0 / size.lifespan_multiplier()) as u8;
        let mut new_age = age.saturating_add(aging_rate);
        
        // Leaves die after adjusted lifespan - transition to withered state
        let max_age = (150.0 * size.lifespan_multiplier()) as u8;
        if new_age >= max_age {
            new_tiles[y][x] = TileType::PlantWithered(0, size);
            return;
        }
        
        // Leaves photosynthesize during day (slow aging)
        if self.is_day() {
            new_age = new_age.saturating_sub(1);
            
            // Healthy leaves can sometimes produce nutrients during the day - rate affected by size
            let nutrient_chance = 0.02 * size.growth_rate_multiplier();
            if new_age < (max_age as u16 * 2 / 3) as u8 && rng.gen_bool(nutrient_chance as f64) {
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
        
        new_tiles[y][x] = TileType::PlantLeaf(new_age, size);
    }
    
    fn update_plant_bud(&self, x: usize, y: usize, age: u8, size: Size, new_tiles: &mut [Vec<TileType>], rng: &mut impl Rng) {
        let aging_rate = (1.0 / size.lifespan_multiplier()) as u8;
        let new_age = age.saturating_add(aging_rate);
        
        // Buds develop into other structures - timing affected by size
        let development_age = (30.0 / size.growth_rate_multiplier()) as u8;
        if new_age >= development_age {
            // Development chances affected by size - larger buds more likely to become stems
            let (stem_chance, flower_chance, leaf_chance) = match size {
                Size::Small => (3, 4, 3),  // More balanced for small
                Size::Medium => (5, 3, 2), // Original distribution
                Size::Large => (7, 2, 1),  // Large buds prefer becoming stems
            };
            let total = stem_chance + flower_chance + leaf_chance;
            let rand_val = rng.gen_range(0..total);
            
            if rand_val < stem_chance {
                new_tiles[y][x] = TileType::PlantStem(0, size);
            } else if rand_val < stem_chance + flower_chance {
                new_tiles[y][x] = TileType::PlantFlower(0, size);
            } else {
                new_tiles[y][x] = TileType::PlantLeaf(0, size);
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
                    if let TileType::PlantStem(_, _) = self.tiles[ny][nx] {
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
        
        new_tiles[y][x] = TileType::PlantBud(new_age, size);
    }
    
    fn update_plant_flower(&self, x: usize, y: usize, age: u8, size: Size, new_tiles: &mut [Vec<TileType>], rng: &mut impl Rng) {
        let aging_rate = (1.0 / size.lifespan_multiplier()) as u8;
        let new_age = age.saturating_add(aging_rate);
        
        // Flowers wither after adjusted lifespan - transition to withered state
        let max_age = (100.0 * size.lifespan_multiplier()) as u8;
        if new_age >= max_age {
            new_tiles[y][x] = TileType::PlantWithered(0, size);
            return;
        }
        
        // Flowers can spread seeds during the day - rate and range affected by size
        let seed_chance = 0.02 * size.growth_rate_multiplier();
        let min_age = (20.0 / size.growth_rate_multiplier()) as u8;
        let max_fertile_age = (80.0 * size.lifespan_multiplier()) as u8;
        
        if self.is_day() && new_age > min_age && new_age < max_fertile_age && rng.gen_bool(seed_chance as f64) {
            // Larger flowers can spread seeds further
            let range = match size {
                Size::Small => 1,
                Size::Medium => 2,
                Size::Large => 3,
            };
            
            let directions = [
                (-range, -1), (-1, -range), (0, -range), (1, -range), (range, -1), 
                (range, 0), (1, 1), (0, range), (-1, 1), (-range, 0)
            ];
            if let Some(&(dx, dy)) = directions.choose(rng) {
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                if nx < self.width && ny < self.height && new_tiles[ny][nx] == TileType::Empty {
                    // Seeds need dirt/sand to grow
                    if ny + 1 < self.height && 
                       (self.tiles[ny + 1][nx] == TileType::Dirt || self.tiles[ny + 1][nx] == TileType::Sand) {
                        // New stems have slight size variation from parent
                        let seed_size = if rng.gen_bool(0.7) { 
                            size 
                        } else { 
                            random_size(rng) 
                        };
                        new_tiles[ny][nx] = TileType::PlantStem(0, seed_size);
                    }
                }
            }
        }
        
        new_tiles[y][x] = TileType::PlantFlower(new_age, size);
    }
    
    fn update_plant_withered(&self, x: usize, y: usize, age: u8, size: Size, new_tiles: &mut [Vec<TileType>], _rng: &mut impl Rng) {
        let aging_rate = 2; // Withered plants decay faster than living ones
        let new_age = age.saturating_add(aging_rate);
        
        // Withered plants become nutrients after 30 ticks
        if new_age >= 30 {
            new_tiles[y][x] = TileType::Nutrient;
            return;
        }
        
        new_tiles[y][x] = TileType::PlantWithered(new_age, size);
    }
    
    fn update_pillbug_head(&self, x: usize, y: usize, age: u8, size: Size, new_tiles: &mut [Vec<TileType>], rng: &mut impl Rng) {
        let aging_rate = (1.0 / size.lifespan_multiplier()) as u8;
        let mut new_age = age.saturating_add(aging_rate);
        
        // Pillbug head dies after adjusted lifespan - transition to decaying state
        let max_age = (180.0 * size.lifespan_multiplier()) as u8;
        if new_age >= max_age {
            new_tiles[y][x] = TileType::PillbugDecaying(0, size);
            return;
        }
        
        // Head is responsible for eating - look for plant parts nearby
        let mut found_food = false;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                if nx < self.width && ny < self.height {
                    let tile = self.tiles[ny][nx];
                    if tile.is_plant() {
                        // Size affects eating preferences and success rates
                        let base_eat_chance = match tile {
                            TileType::PlantLeaf(_, _) => 0.15,     // Prefer leaves
                            TileType::PlantFlower(_, _) => 0.12,   // Like flowers
                            TileType::PlantBud(_, _) => 0.08,      // Eat buds sometimes
                            TileType::PlantStem(_, _) => 0.03,     // Rarely eat stems
                            TileType::PlantWithered(_, _) => 0.20, // Love withered plants (easier to eat)
                            _ => 0.0,
                        };
                        
                        // Large pillbugs can eat plants more easily, especially large plants
                        let size_modifier = match (size, tile.get_size()) {
                            (Size::Large, Some(Size::Small)) => 1.5,  // Large bugs love small plants
                            (Size::Large, Some(Size::Medium)) => 1.2,
                            (Size::Large, Some(Size::Large)) => 1.0,
                            (Size::Medium, Some(Size::Small)) => 1.3,
                            (Size::Medium, Some(Size::Medium)) => 1.0,
                            (Size::Medium, Some(Size::Large)) => 0.7,  // Hard to eat large plants
                            (Size::Small, Some(Size::Small)) => 1.0,
                            (Size::Small, Some(Size::Medium)) => 0.6,  // Small bugs struggle with medium plants
                            (Size::Small, Some(Size::Large)) => 0.3,   // Very hard to eat large plants
                            _ => 1.0,
                        };
                        
                        let eat_chance = base_eat_chance * size_modifier;
                        if rng.gen_bool(eat_chance) {
                            new_tiles[ny][nx] = TileType::Nutrient; // Plant part becomes nutrient
                            new_age = new_age.saturating_sub(10); // Food slows aging significantly
                            found_food = true;
                            
                            // When head finds food, try to reproduce by spawning a new pillbug nearby
                            let repro_chance = 0.03 * size.growth_rate_multiplier();
                            if rng.gen_bool(repro_chance as f64) {
                                self.try_spawn_pillbug(x, y, size, new_tiles, rng);
                            }
                            break;
                        }
                    }
                }
            }
            if found_food { break; }
        }
        
        // Without food, age faster (starve) - size affects starvation rate
        if !found_food {
            let starvation_rate = match size {
                Size::Small => 1,   // Small bugs starve slower (need less food)
                Size::Medium => 2,  // Normal starvation rate
                Size::Large => 3,   // Large bugs starve faster (need more food)
            };
            new_age = new_age.saturating_add(starvation_rate);
        }
        
        // Head coordinates movement for the whole pillbug - movement rate varies by size
        let movement_chance = match size {
            Size::Small => 0.12,  // Small bugs move more frequently
            Size::Medium => 0.08, // Normal movement rate
            Size::Large => 0.05,  // Large bugs move slower
        };
        if rng.gen_bool(movement_chance) {
            self.try_move_pillbug(x, y, new_age, size, new_tiles, rng);
        } else {
            new_tiles[y][x] = TileType::PillbugHead(new_age, size);
        }
    }
    
    fn update_pillbug_body(&self, x: usize, y: usize, age: u8, size: Size, new_tiles: &mut [Vec<TileType>], _rng: &mut impl Rng) {
        let aging_rate = (1.0 / size.lifespan_multiplier()) as u8;
        let new_age = age.saturating_add(aging_rate);
        
        // Pillbug body dies after adjusted lifespan - transition to decaying state
        let max_age = (180.0 * size.lifespan_multiplier()) as u8;
        if new_age >= max_age {
            new_tiles[y][x] = TileType::PillbugDecaying(0, size);
            return;
        }
        
        new_tiles[y][x] = TileType::PillbugBody(new_age, size);
    }
    
    fn update_pillbug_legs(&self, x: usize, y: usize, age: u8, size: Size, new_tiles: &mut [Vec<TileType>], _rng: &mut impl Rng) {
        let aging_rate = (1.0 / size.lifespan_multiplier()) as u8;
        let new_age = age.saturating_add(aging_rate);
        
        // Pillbug legs die after adjusted lifespan - transition to decaying state
        let max_age = (180.0 * size.lifespan_multiplier()) as u8;
        if new_age >= max_age {
            new_tiles[y][x] = TileType::PillbugDecaying(0, size);
            return;
        }
        
        new_tiles[y][x] = TileType::PillbugLegs(new_age, size);
    }
    
    fn update_pillbug_decaying(&self, x: usize, y: usize, age: u8, size: Size, new_tiles: &mut [Vec<TileType>], _rng: &mut impl Rng) {
        let aging_rate = 1; // Decaying pillbugs decompose at normal rate
        let new_age = age.saturating_add(aging_rate);
        
        // Decaying pillbugs become nutrients after 20 ticks
        if new_age >= 20 {
            new_tiles[y][x] = TileType::Nutrient;
            return;
        }
        
        new_tiles[y][x] = TileType::PillbugDecaying(new_age, size);
    }
    
    fn try_spawn_pillbug(&self, x: usize, y: usize, parent_size: Size, new_tiles: &mut [Vec<TileType>], rng: &mut impl Rng) {
        let directions = [(-3, 0), (3, 0), (0, -3), (0, 3), (-2, -2), (2, 2), (-2, 2), (2, -2)];
        if let Some(&(dx, dy)) = directions.choose(rng) {
            let spawn_x = (x as i32 + dx) as usize;
            let spawn_y = (y as i32 + dy) as usize;
            
            if spawn_x < self.width && spawn_y < self.height && 
               new_tiles[spawn_y][spawn_x] == TileType::Empty {
                // Baby pillbugs inherit size with some variation
                let baby_size = if rng.gen_bool(0.8) { 
                    parent_size 
                } else { 
                    random_size(rng) 
                };
                
                // Create baby pillbug head, and try to create body/legs nearby
                new_tiles[spawn_y][spawn_x] = TileType::PillbugHead(0, baby_size);
                
                // Try to spawn body nearby
                let body_dirs = [(-1, 0), (1, 0), (0, 1), (0, -1)];
                if let Some(&(bdx, bdy)) = body_dirs.choose(rng) {
                    let body_x = (spawn_x as i32 + bdx) as usize;
                    let body_y = (spawn_y as i32 + bdy) as usize;
                    if body_x < self.width && body_y < self.height && new_tiles[body_y][body_x] == TileType::Empty {
                        new_tiles[body_y][body_x] = TileType::PillbugBody(0, baby_size);
                        
                        // Try to spawn legs near body
                        let leg_dirs = [(-1, 0), (1, 0), (0, 1), (0, -1)];
                        if let Some(&(ldx, ldy)) = leg_dirs.choose(rng) {
                            let legs_x = (body_x as i32 + ldx) as usize;
                            let legs_y = (body_y as i32 + ldy) as usize;
                            if legs_x < self.width && legs_y < self.height && new_tiles[legs_y][legs_x] == TileType::Empty {
                                new_tiles[legs_y][legs_x] = TileType::PillbugLegs(0, baby_size);
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn try_move_pillbug(&self, x: usize, y: usize, age: u8, size: Size, new_tiles: &mut [Vec<TileType>], rng: &mut impl Rng) {
        // Find connected body parts nearby
        let mut body_parts = Vec::new();
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                if nx < self.width && ny < self.height {
                    match self.tiles[ny][nx] {
                        TileType::PillbugBody(body_age, body_size) if body_size == size => {
                            body_parts.push((nx, ny, TileType::PillbugBody(body_age, body_size)));
                        }
                        TileType::PillbugLegs(legs_age, legs_size) if legs_size == size => {
                            body_parts.push((nx, ny, TileType::PillbugLegs(legs_age, legs_size)));
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Try to move head to a new position
        let directions = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
        if let Some(&(dx, dy)) = directions.choose(rng) {
            let head_x = (x as i32 + dx) as usize;
            let head_y = (y as i32 + dy) as usize;
            
            if head_x < self.width && head_y < self.height && new_tiles[head_y][head_x] == TileType::Empty {
                // Move head to new position
                new_tiles[y][x] = TileType::Empty;
                new_tiles[head_y][head_x] = TileType::PillbugHead(age, size);
                
                // Try to move body parts to follow head in a chain
                let mut previous_pos = (head_x, head_y);
                
                for (part_x, part_y, part_tile) in body_parts {
                    // Find best position for this body part (close to previous position)
                    let mut best_pos = None;
                    let mut best_distance = f32::INFINITY;
                    
                    for check_dy in -1..=1 {
                        for check_dx in -1..=1 {
                            let check_x = (previous_pos.0 as i32 + check_dx) as usize;
                            let check_y = (previous_pos.1 as i32 + check_dy) as usize;
                            
                            if check_x < self.width && check_y < self.height {
                                let is_empty = if (check_x, check_y) == (part_x, part_y) {
                                    true // Current position of part
                                } else {
                                    new_tiles[check_y][check_x] == TileType::Empty
                                };
                                
                                if is_empty {
                                    let distance = ((check_x as f32 - previous_pos.0 as f32).powi(2) + 
                                                   (check_y as f32 - previous_pos.1 as f32).powi(2)).sqrt();
                                    if distance < best_distance {
                                        best_distance = distance;
                                        best_pos = Some((check_x, check_y));
                                    }
                                }
                            }
                        }
                    }
                    
                    // Move body part to best position if different from current
                    if let Some((new_x, new_y)) = best_pos {
                        if (new_x, new_y) != (part_x, part_y) {
                            new_tiles[part_y][part_x] = TileType::Empty;
                            new_tiles[new_y][new_x] = part_tile;
                            previous_pos = (new_x, new_y);
                        } else {
                            // Keep part where it is
                            previous_pos = (part_x, part_y);
                        }
                    } else {
                        // Can't move part, keep it in place
                        previous_pos = (part_x, part_y);
                    }
                }
            } else {
                // Can't move head, keep all parts in place
                new_tiles[y][x] = TileType::PillbugHead(age, size);
            }
        } else {
            new_tiles[y][x] = TileType::PillbugHead(age, size);
        }
    }
    
    fn count_entities(&self) -> (usize, usize) {
        let mut plant_count = 0;
        let mut pillbug_count = 0;
        
        for y in 0..self.height {
            for x in 0..self.width {
                match self.tiles[y][x] {
                    TileType::PlantStem(_, _) => plant_count += 1,
                    TileType::PillbugHead(_, _) => pillbug_count += 1,
                    _ => {}
                }
            }
        }
        
        (plant_count, pillbug_count)
    }
    
    fn spawn_entities(&mut self) {
        let (plant_count, pillbug_count) = self.count_entities();
        let mut rng = rand::thread_rng();
        
        // Spawn plants if under 3
        if plant_count < 3 {
            for _ in 0..(3 - plant_count) {
                self.spawn_plant_at_top(&mut rng);
            }
        }
        
        // Spawn pillbugs if under 3
        if pillbug_count < 3 {
            for _ in 0..(3 - pillbug_count) {
                self.spawn_pillbug_at_top(&mut rng);
            }
        }
    }
    
    fn spawn_plant_at_top(&mut self, rng: &mut impl Rng) {
        // Try to find an empty spot at the top of the world
        for _ in 0..50 { // Limit attempts to avoid infinite loop
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..5); // Top 5 rows
            
            if self.tiles[y][x] == TileType::Empty {
                let plant_size = random_size(rng);
                self.tiles[y][x] = TileType::PlantStem(rng.gen_range(5..30), plant_size);
                return;
            }
        }
    }
    
    fn spawn_pillbug_at_top(&mut self, rng: &mut impl Rng) {
        // Try to find an empty spot at the top of the world for the pillbug head
        for _ in 0..50 { // Limit attempts to avoid infinite loop
            let x = rng.gen_range(1..self.width - 1);
            let y = rng.gen_range(0..5); // Top 5 rows
            
            if self.tiles[y][x] == TileType::Empty {
                let pillbug_size = random_size(rng);
                let age = rng.gen_range(10..50);
                
                // Create pillbug head
                self.tiles[y][x] = TileType::PillbugHead(age, pillbug_size);
                
                // Try to place body nearby
                let directions = [(-1, 0), (1, 0), (0, 1), (0, -1)];
                if let Some(&(dx, dy)) = directions.choose(rng) {
                    let body_x = (x as i32 + dx) as usize;
                    let body_y = (y as i32 + dy) as usize;
                    if body_x < self.width && body_y < self.height && self.tiles[body_y][body_x] == TileType::Empty {
                        self.tiles[body_y][body_x] = TileType::PillbugBody(age, pillbug_size);
                        
                        // Try to place legs adjacent to body
                        let leg_directions = [(-1, 0), (1, 0), (0, 1), (0, -1)];
                        if let Some(&(ldx, ldy)) = leg_directions.choose(rng) {
                            let legs_x = (body_x as i32 + ldx) as usize;
                            let legs_y = (body_y as i32 + ldy) as usize;
                            if legs_x < self.width && legs_y < self.height && self.tiles[legs_y][legs_x] == TileType::Empty {
                                self.tiles[legs_y][legs_x] = TileType::PillbugLegs(age, pillbug_size);
                            }
                        }
                    }
                }
                return;
            }
        }
    }
    
    fn is_day(&self) -> bool {
        self.day_cycle.sin() > 0.0
    }
}

impl fmt::Display for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "{}", self.tiles[y][x].to_char())?;
            }
            writeln!(f)?;
        }
        writeln!(f, "Tick: {}", self.tick)?;
        writeln!(f, "Day/Night: {}", if self.is_day() { "Day" } else { "Night" })?;
        writeln!(f, "Rain intensity: {:.2}", self.rain_intensity)?;
        Ok(())
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
    let args: Vec<String> = env::args().collect();
    
    // Parse command line arguments
    let mut sim_ticks: Option<u64> = None;
    let mut output_file: Option<String> = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            arg if arg.starts_with("--sim-ticks=") => {
                let ticks_str = arg.strip_prefix("--sim-ticks=").unwrap();
                sim_ticks = Some(ticks_str.parse().map_err(|_| "Invalid --sim-ticks value")?);
            }
            arg if arg.starts_with("--output-file=") => {
                let file_str = arg.strip_prefix("--output-file=").unwrap();
                output_file = Some(file_str.to_string());
            }
            "--help" | "-h" => {
                println!("Pillbug Plants Simulation");
                println!("Usage: {} [options]", args[0]);
                println!("Options:");
                println!("  --sim-ticks=N    Run simulation for N ticks and exit");
                println!("  --output-file=F  Save simulation output to file F");
                println!("  --help, -h       Show this help message");
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
        i += 1;
    }
    
    // Run in simulation mode if --sim-ticks is specified
    if let Some(ticks) = sim_ticks {
        return run_simulation(ticks, output_file);
    }
    
    // Set up panic hook to restore terminal state
    std::panic::set_hook(Box::new(|panic_info| {
        // Try to restore terminal state
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        
        eprintln!("{}", panic_info);
    }));
    
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

fn run_simulation(ticks: u64, output_file: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Create a world with fixed dimensions for consistency
    let world_width = 80;
    let world_height = 40;
    let mut world = World::new(world_width, world_height);
    
    println!("Running simulation for {} ticks...", ticks);
    
    // Run simulation
    for tick in 0..ticks {
        world.update();
        
        // Print progress every 100 ticks
        if tick % 100 == 0 || tick == ticks - 1 {
            println!("Progress: {}/{} ticks", tick + 1, ticks);
        }
    }
    
    let final_state = world.to_string();
    
    // Output results
    if let Some(file_path) = output_file {
        let mut file = File::create(&file_path)?;
        write!(file, "{}", final_state)?;
        println!("Simulation results saved to: {}", file_path);
    } else {
        println!("Final simulation state:");
        print!("{}", final_state);
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
            Line::from("PLANTS (now with size variations!):"),
            Line::from(vec![
                Span::styled("i|║", Style::default().fg(Color::Rgb(80, 200, 60))),
                Span::raw(" = Plant Stem (small/med/large)")
            ]),
            Line::from(vec![
                Span::styled("lLŁ", Style::default().fg(Color::Green)),
                Span::raw(" = Plant Leaf (small/med/large)")
            ]),
            Line::from(vec![
                Span::styled("°oO", Style::default().fg(Color::Rgb(200, 100, 0))),
                Span::raw(" = Plant Bud (small/med/large)")
            ]),
            Line::from(vec![
                Span::styled("·*✱", Style::default().fg(Color::Rgb(255, 150, 200))),
                Span::raw(" = Plant Flower (small/med/large)")
            ]),
            Line::from(vec![
                Span::styled("x", Style::default().fg(Color::Rgb(100, 50, 0))),
                Span::raw(" = Plant Withered (gradual decay)")
            ]),
            Line::from("  - Size affects: lifespan, growth rate, spread"),
            Line::from("  - Large: live longer, grow/reproduce slower"),
            Line::from("  - Small: live shorter, grow/reproduce faster"),
            Line::from("  - Large flowers spread seeds farther"),
            Line::from(""),
            Line::from("PILLBUGS (multi-segment with sizes!):"),
            Line::from(vec![
                Span::styled("ó@●", Style::default().fg(Color::Rgb(140, 120, 110))),
                Span::raw(" = Pillbug Head (small/med/large)")
            ]),
            Line::from(vec![
                Span::styled("oO●", Style::default().fg(Color::Gray)),
                Span::raw(" = Pillbug Body (small/med/large)")
            ]),
            Line::from(vec![
                Span::styled("vwW", Style::default().fg(Color::Rgb(110, 120, 140))),
                Span::raw(" = Pillbug Legs (small/med/large)")
            ]),
            Line::from(vec![
                Span::styled("░", Style::default().fg(Color::Rgb(80, 26, 40))),
                Span::raw(" = Pillbug Decaying (gradual decay)")
            ]),
            Line::from("  - Size affects: movement, eating, lifespan"),
            Line::from("  - Large: eat better, move slower, starve faster"),
            Line::from("  - Small: move faster, struggle with big plants"),
            Line::from("  - Size inheritance with some variation"),
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