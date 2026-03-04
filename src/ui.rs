// TUI rendering: map with ping rays, stats panel, log panel

use crate::app::App;
use crate::config::Config;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, canvas::{Canvas, Map, MapResolution}};

fn latency_color(ms: f64, cfg: &Config) -> Color {
    if ms < cfg.green_below_ms {
        Color::Green
    } else if ms < cfg.yellow_below_ms {
        Color::Yellow
    } else {
        Color::Red
    }
}

pub fn draw(f: &mut Frame, app: &App, cfg: &Config) {
    // top: map + stats side by side; bottom: log
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(15), Constraint::Length(8)])
        .split(f.area());

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(outer[0]);

    draw_map(f, top[0], app, cfg);
    draw_stats(f, top[1], app, cfg);
    draw_log(f, outer[1], app);
}

fn draw_map(f: &mut Frame, area: Rect, app: &App, cfg: &Config) {
    let canvas = Canvas::default()
        .block(Block::default().title(" Map ").borders(Borders::ALL))
        .x_bounds([cfg.map_lon_min, cfg.map_lon_max])
        .y_bounds([cfg.map_lat_min, cfg.map_lat_max])
        .paint(|ctx| {
            // background map
            ctx.draw(&Map {
                resolution: MapResolution::High,
                color: Color::DarkGray,
            });

            // draw rays from user to each target
            for (i, target) in app.group.targets.iter().enumerate() {
                let color = match &app.latest[i] {
                    Some(r) if r.success => latency_color(r.latency_ms, cfg),
                    Some(_) => Color::DarkGray,
                    None => Color::DarkGray,
                };
                ctx.draw(&ratatui::widgets::canvas::Line {
                    x1: cfg.user_lon,
                    y1: cfg.user_lat,
                    x2: target.lon,
                    y2: target.lat,
                    color,
                });
            }

            // draw target points
            for (i, target) in app.group.targets.iter().enumerate() {
                let color = match &app.latest[i] {
                    Some(r) if r.success => latency_color(r.latency_ms, cfg),
                    Some(_) => Color::Red,
                    None => Color::DarkGray,
                };
                ctx.print(target.lon, target.lat, Span::styled("●", Style::default().fg(color)));
            }

            // draw user location
            ctx.print(cfg.user_lon, cfg.user_lat, Span::styled("◆", Style::default().fg(Color::Cyan)));
        });

    f.render_widget(canvas, area);
}

fn draw_stats(f: &mut Frame, area: Rect, app: &App, cfg: &Config) {
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(format!(" {}", app.group.name)).bold());
    lines.push(Line::from(""));

    if let Some(med) = app.median_latency() {
        lines.push(Line::from(format!(" median:  {med:.1}ms")));
    }
    if let Some(jit) = app.jitter() {
        lines.push(Line::from(format!(" jitter:  {jit:.1}ms")));
    }
    let loss = app.loss_count();
    let total = app.group.targets.len();
    lines.push(Line::from(format!(" loss:    {loss}/{total}")));
    lines.push(Line::from(""));

    let fastest = app.fastest_index();

    // per-host latency list
    for (i, target) in app.group.targets.iter().enumerate() {
        let marker = if fastest == Some(i) { "<" } else { "" };
        let text = match &app.latest[i] {
            Some(r) if r.success => {
                let c = latency_color(r.latency_ms, cfg);
                Line::from(vec![
                    Span::raw(format!(" {:<12} ", target.name)),
                    Span::styled(format!("{:>6.1}ms", r.latency_ms), Style::default().fg(c)),
                    Span::styled(format!(" {marker}"), Style::default().fg(Color::Cyan)),
                ])
            }
            Some(_) => Line::from(vec![
                Span::raw(format!(" {:<12} ", target.name)),
                Span::styled("  FAIL", Style::default().fg(Color::Red)),
            ]),
            None => Line::from(vec![
                Span::raw(format!(" {:<12} ", target.name)),
                Span::styled("   ---", Style::default().fg(Color::DarkGray)),
            ]),
        };
        lines.push(text);
    }

    let para = Paragraph::new(lines)
        .block(Block::default().title(" Stats ").borders(Borders::ALL));
    f.render_widget(para, area);
}

fn draw_log(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app.log.iter().rev()
        .take(area.height.saturating_sub(2) as usize)
        .map(|s| ListItem::new(format!(" {s}")))
        .collect();

    let list = List::new(items)
        .block(Block::default().title(" Log  [q] quit ").borders(Borders::ALL));
    f.render_widget(list, area);
}
