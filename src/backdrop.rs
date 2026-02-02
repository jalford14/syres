use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::canvas::{Canvas, Line as CanvasLine, Rectangle as CanvasRect},
};

use crate::theme;

// ── Backdrop palette — dim, atmospheric tones ───────────────────────
const STRUCTURE: Color = Color::Rgb(45, 41, 36);
const STRUCTURE_DIM: Color = Color::Rgb(38, 35, 31);
const FURNITURE: Color = Color::Rgb(52, 47, 41);
const FURNITURE_DIM: Color = Color::Rgb(42, 38, 34);
const LIGHT_GLOW: Color = Color::Rgb(80, 60, 0);
const GLASS: Color = Color::Rgb(38, 44, 52);
const PLANT: Color = Color::Rgb(48, 58, 40);
const ACCENT: Color = Color::Rgb(60, 50, 35);
const FLOOR_PLANK: Color = Color::Rgb(40, 36, 30);
const FLOOR_DIM: Color = Color::Rgb(35, 32, 27);
const LABEL_COLOR: Color = Color::Rgb(55, 48, 38);

/// Render a 2D line-drawing of a coworking café interior as a permanent
/// backdrop behind all UI views.
pub fn render_backdrop(frame: &mut Frame, area: Rect) {
    let canvas = Canvas::default()
        .x_bounds([0.0, 200.0])
        .y_bounds([0.0, 100.0])
        .marker(Marker::Braille)
        .background_color(theme::WARM_BLACK)
        .paint(|ctx| {
            // ── Floor ───────────────────────────────────────────
            ctx.draw(&CanvasLine {
                x1: 2.0,
                y1: 15.0,
                x2: 198.0,
                y2: 15.0,
                color: STRUCTURE,
            });
            // Wood plank lines
            for &y in &[3.0, 6.0, 9.0, 12.0] {
                let c = if y > 8.0 { FLOOR_PLANK } else { FLOOR_DIM };
                ctx.draw(&CanvasLine {
                    x1: 4.0,
                    y1: y,
                    x2: 196.0,
                    y2: y,
                    color: c,
                });
            }
            // Plank joints
            for &x in &[30.0, 60.0, 90.0, 120.0, 150.0, 180.0] {
                ctx.draw(&CanvasLine {
                    x1: x,
                    y1: 1.0,
                    x2: x,
                    y2: 14.0,
                    color: FLOOR_DIM,
                });
            }

            // ── Ceiling ─────────────────────────────────────────
            ctx.draw(&CanvasLine {
                x1: 2.0,
                y1: 90.0,
                x2: 198.0,
                y2: 90.0,
                color: STRUCTURE,
            });
            // Crown molding
            ctx.draw(&CanvasLine {
                x1: 2.0,
                y1: 88.0,
                x2: 198.0,
                y2: 88.0,
                color: STRUCTURE_DIM,
            });
            // Exposed beam
            ctx.draw(&CanvasLine {
                x1: 50.0,
                y1: 89.0,
                x2: 150.0,
                y2: 89.0,
                color: STRUCTURE_DIM,
            });

            // ── Walls ───────────────────────────────────────────
            ctx.draw(&CanvasLine {
                x1: 2.0,
                y1: 15.0,
                x2: 2.0,
                y2: 90.0,
                color: STRUCTURE,
            });
            ctx.draw(&CanvasLine {
                x1: 198.0,
                y1: 15.0,
                x2: 198.0,
                y2: 90.0,
                color: STRUCTURE,
            });

            // ── Pendant Lights ──────────────────────────────────
            for &x in &[40.0, 80.0, 125.0, 170.0] {
                // Wire
                ctx.draw(&CanvasLine {
                    x1: x,
                    y1: 90.0,
                    x2: x,
                    y2: 77.0,
                    color: STRUCTURE_DIM,
                });
                // Shade
                ctx.draw(&CanvasLine {
                    x1: x - 5.0,
                    y1: 77.0,
                    x2: x + 5.0,
                    y2: 77.0,
                    color: LIGHT_GLOW,
                });
                ctx.draw(&CanvasLine {
                    x1: x - 5.0,
                    y1: 77.0,
                    x2: x - 2.0,
                    y2: 74.0,
                    color: LIGHT_GLOW,
                });
                ctx.draw(&CanvasLine {
                    x1: x + 5.0,
                    y1: 77.0,
                    x2: x + 2.0,
                    y2: 74.0,
                    color: LIGHT_GLOW,
                });
                // Bulb
                ctx.draw(&CanvasLine {
                    x1: x - 1.0,
                    y1: 75.0,
                    x2: x + 1.0,
                    y2: 75.0,
                    color: LIGHT_GLOW,
                });
            }

            // ── Bookshelf (far left wall) ───────────────────────
            ctx.draw(&CanvasRect {
                x: 8.0,
                y: 30.0,
                width: 22.0,
                height: 52.0,
                color: FURNITURE,
            });
            // Shelf dividers
            for &y in &[40.0, 50.0, 60.0, 70.0] {
                ctx.draw(&CanvasLine {
                    x1: 8.0,
                    y1: y,
                    x2: 30.0,
                    y2: y,
                    color: FURNITURE,
                });
            }
            // Book spines
            for &shelf_y in &[31.0, 41.0, 51.0, 61.0, 71.0] {
                let mut bx = 10.0;
                while bx < 28.0 {
                    let bh = 6.0 + ((bx * 7.3) % 3.0);
                    ctx.draw(&CanvasLine {
                        x1: bx,
                        y1: shelf_y,
                        x2: bx,
                        y2: shelf_y + bh,
                        color: ACCENT,
                    });
                    bx += 2.2;
                }
            }

            // ── Phone Booth (Switchyards signature) ─────────────
            ctx.draw(&CanvasRect {
                x: 36.0,
                y: 15.0,
                width: 14.0,
                height: 62.0,
                color: FURNITURE,
            });
            // Door frame
            ctx.draw(&CanvasLine {
                x1: 40.0,
                y1: 15.0,
                x2: 40.0,
                y2: 55.0,
                color: FURNITURE_DIM,
            });
            ctx.draw(&CanvasLine {
                x1: 46.0,
                y1: 15.0,
                x2: 46.0,
                y2: 55.0,
                color: FURNITURE_DIM,
            });
            // Glass panel
            ctx.draw(&CanvasRect {
                x: 41.0,
                y: 35.0,
                width: 4.0,
                height: 18.0,
                color: GLASS,
            });

            // ── Coffee Counter ──────────────────────────────────
            ctx.draw(&CanvasRect {
                x: 56.0,
                y: 15.0,
                width: 38.0,
                height: 14.0,
                color: FURNITURE,
            });
            // Counter top edge
            ctx.draw(&CanvasLine {
                x1: 56.0,
                y1: 29.0,
                x2: 94.0,
                y2: 29.0,
                color: FURNITURE,
            });
            // Espresso machine
            ctx.draw(&CanvasRect {
                x: 60.0,
                y: 29.0,
                width: 8.0,
                height: 7.0,
                color: ACCENT,
            });
            // Steam wand
            ctx.draw(&CanvasLine {
                x1: 64.0,
                y1: 36.0,
                x2: 64.0,
                y2: 39.0,
                color: STRUCTURE_DIM,
            });
            // Cups on counter
            ctx.draw(&CanvasRect {
                x: 72.0,
                y: 29.0,
                width: 3.0,
                height: 3.0,
                color: ACCENT,
            });
            ctx.draw(&CanvasRect {
                x: 77.0,
                y: 29.0,
                width: 3.0,
                height: 3.0,
                color: ACCENT,
            });

            // Bar stools
            for &sx in &[62.0, 72.0, 82.0] {
                // Pedestal
                ctx.draw(&CanvasLine {
                    x1: sx,
                    y1: 15.0,
                    x2: sx,
                    y2: 21.0,
                    color: FURNITURE_DIM,
                });
                // Seat
                ctx.draw(&CanvasLine {
                    x1: sx - 2.5,
                    y1: 21.0,
                    x2: sx + 2.5,
                    y2: 21.0,
                    color: FURNITURE_DIM,
                });
                // Backrest
                ctx.draw(&CanvasLine {
                    x1: sx - 1.5,
                    y1: 21.0,
                    x2: sx - 1.5,
                    y2: 26.0,
                    color: FURNITURE_DIM,
                });
            }

            // Small plant on counter
            ctx.draw(&CanvasRect {
                x: 88.0,
                y: 29.0,
                width: 3.0,
                height: 3.0,
                color: ACCENT,
            });
            ctx.draw(&CanvasLine {
                x1: 89.5,
                y1: 32.0,
                x2: 87.0,
                y2: 38.0,
                color: PLANT,
            });
            ctx.draw(&CanvasLine {
                x1: 89.5,
                y1: 32.0,
                x2: 92.0,
                y2: 37.0,
                color: PLANT,
            });
            ctx.draw(&CanvasLine {
                x1: 89.5,
                y1: 34.0,
                x2: 86.0,
                y2: 40.0,
                color: PLANT,
            });

            // ── Work Tables ─────────────────────────────────────
            // Table 1
            draw_table(ctx, 102.0, 15.0);
            // Table 2
            draw_table(ctx, 135.0, 15.0);

            // ── Wall Art ────────────────────────────────────────
            // Frame 1
            ctx.draw(&CanvasRect {
                x: 60.0,
                y: 50.0,
                width: 16.0,
                height: 20.0,
                color: ACCENT,
            });
            // Inner mat
            ctx.draw(&CanvasRect {
                x: 62.0,
                y: 52.0,
                width: 12.0,
                height: 16.0,
                color: FURNITURE_DIM,
            });
            // Frame 2
            ctx.draw(&CanvasRect {
                x: 82.0,
                y: 53.0,
                width: 12.0,
                height: 16.0,
                color: ACCENT,
            });
            ctx.draw(&CanvasRect {
                x: 84.0,
                y: 55.0,
                width: 8.0,
                height: 12.0,
                color: FURNITURE_DIM,
            });

            // ── Clock ───────────────────────────────────────────
            ctx.draw(&CanvasRect {
                x: 108.0,
                y: 60.0,
                width: 8.0,
                height: 8.0,
                color: ACCENT,
            });
            // Hour hand
            ctx.draw(&CanvasLine {
                x1: 112.0,
                y1: 64.0,
                x2: 112.0,
                y2: 67.0,
                color: LIGHT_GLOW,
            });
            // Minute hand
            ctx.draw(&CanvasLine {
                x1: 112.0,
                y1: 64.0,
                x2: 114.5,
                y2: 64.0,
                color: LIGHT_GLOW,
            });

            // ── Windows (right wall) ────────────────────────────
            // Window 1
            ctx.draw(&CanvasRect {
                x: 148.0,
                y: 35.0,
                width: 20.0,
                height: 38.0,
                color: GLASS,
            });
            ctx.draw(&CanvasLine {
                x1: 158.0,
                y1: 35.0,
                x2: 158.0,
                y2: 73.0,
                color: STRUCTURE_DIM,
            });
            ctx.draw(&CanvasLine {
                x1: 148.0,
                y1: 54.0,
                x2: 168.0,
                y2: 54.0,
                color: STRUCTURE_DIM,
            });
            // Sill
            ctx.draw(&CanvasLine {
                x1: 146.0,
                y1: 35.0,
                x2: 170.0,
                y2: 35.0,
                color: FURNITURE,
            });

            // Window 2
            ctx.draw(&CanvasRect {
                x: 174.0,
                y: 35.0,
                width: 18.0,
                height: 38.0,
                color: GLASS,
            });
            ctx.draw(&CanvasLine {
                x1: 183.0,
                y1: 35.0,
                x2: 183.0,
                y2: 73.0,
                color: STRUCTURE_DIM,
            });
            ctx.draw(&CanvasLine {
                x1: 174.0,
                y1: 54.0,
                x2: 192.0,
                y2: 54.0,
                color: STRUCTURE_DIM,
            });
            // Sill
            ctx.draw(&CanvasLine {
                x1: 172.0,
                y1: 35.0,
                x2: 194.0,
                y2: 35.0,
                color: FURNITURE,
            });

            // ── Large Corner Plant ──────────────────────────────
            // Pot
            ctx.draw(&CanvasRect {
                x: 188.0,
                y: 15.0,
                width: 8.0,
                height: 7.0,
                color: ACCENT,
            });
            // Foliage
            ctx.draw(&CanvasLine {
                x1: 192.0,
                y1: 22.0,
                x2: 187.0,
                y2: 34.0,
                color: PLANT,
            });
            ctx.draw(&CanvasLine {
                x1: 192.0,
                y1: 22.0,
                x2: 197.0,
                y2: 33.0,
                color: PLANT,
            });
            ctx.draw(&CanvasLine {
                x1: 192.0,
                y1: 25.0,
                x2: 185.0,
                y2: 38.0,
                color: PLANT,
            });
            ctx.draw(&CanvasLine {
                x1: 192.0,
                y1: 25.0,
                x2: 199.0,
                y2: 36.0,
                color: PLANT,
            });
            ctx.draw(&CanvasLine {
                x1: 192.0,
                y1: 28.0,
                x2: 189.0,
                y2: 40.0,
                color: PLANT,
            });
            ctx.draw(&CanvasLine {
                x1: 192.0,
                y1: 28.0,
                x2: 195.0,
                y2: 40.0,
                color: PLANT,
            });

            // ── Area Rug ────────────────────────────────────────
            ctx.draw(&CanvasRect {
                x: 96.0,
                y: 15.5,
                width: 60.0,
                height: 0.5,
                color: ACCENT,
            });

            // ── Signage ─────────────────────────────────────────
            ctx.print(
                100.0,
                95.0,
                Line::from(Span::styled(
                    "S W I T C H Y A R D S",
                    Style::default().fg(LABEL_COLOR),
                )),
            );
        });

    frame.render_widget(canvas, area);
}

/// Draw a work table with two chairs at the given position.
fn draw_table(ctx: &mut ratatui::widgets::canvas::Context, x: f64, floor_y: f64) {
    let top_y = floor_y + 10.0;

    // Tabletop
    ctx.draw(&CanvasLine {
        x1: x,
        y1: top_y,
        x2: x + 18.0,
        y2: top_y,
        color: FURNITURE,
    });
    // Legs
    ctx.draw(&CanvasLine {
        x1: x + 3.0,
        y1: floor_y,
        x2: x + 3.0,
        y2: top_y,
        color: FURNITURE_DIM,
    });
    ctx.draw(&CanvasLine {
        x1: x + 15.0,
        y1: floor_y,
        x2: x + 15.0,
        y2: top_y,
        color: FURNITURE_DIM,
    });
    // Left chair
    ctx.draw(&CanvasRect {
        x: x - 4.0,
        y: floor_y + 1.0,
        width: 5.0,
        height: 14.0,
        color: FURNITURE_DIM,
    });
    // Right chair
    ctx.draw(&CanvasRect {
        x: x + 17.0,
        y: floor_y + 1.0,
        width: 5.0,
        height: 14.0,
        color: FURNITURE_DIM,
    });
    // Laptop / book on table
    ctx.draw(&CanvasRect {
        x: x + 5.0,
        y: top_y,
        width: 7.0,
        height: 2.0,
        color: ACCENT,
    });
}
