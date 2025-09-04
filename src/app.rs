use std::io;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use crate::world::World;

pub struct App {
    pub world: World,
    pub show_taxonomy: bool,
}

impl App {
    pub fn new(width: usize, height: usize) -> Self {
        App {
            world: World::new(width, height),
            show_taxonomy: false,
        }
    }
    
    pub fn tick(&mut self) {
        self.world.update();
    }
}

pub fn run_app<B: Backend>(
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

pub fn ui(f: &mut Frame, app: &App) {
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