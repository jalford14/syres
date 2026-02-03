use ratatui::{
    Frame,
    layout::Rect,
    style::Color,
    symbols::Marker,
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
                    y2: 82.0,
                    color: STRUCTURE_DIM,
                });
                // Shade
                ctx.draw(&CanvasLine {
                    x1: x - 5.0,
                    y1: 82.0,
                    x2: x + 5.0,
                    y2: 82.0,
                    color: LIGHT_GLOW,
                });
                ctx.draw(&CanvasLine {
                    x1: x - 5.0,
                    y1: 82.0,
                    x2: x - 2.0,
                    y2: 79.0,
                    color: LIGHT_GLOW,
                });
                ctx.draw(&CanvasLine {
                    x1: x + 5.0,
                    y1: 82.0,
                    x2: x + 2.0,
                    y2: 79.0,
                    color: LIGHT_GLOW,
                });
                // Bulb
                ctx.draw(&CanvasLine {
                    x1: x - 1.0,
                    y1: 80.0,
                    x2: x + 1.0,
                    y2: 80.0,
                    color: LIGHT_GLOW,
                });
            }

            // ── Coffee Counter (far left) ─────────────────────
            ctx.draw(&CanvasRect {
                x: 10.0,
                y: 15.0,
                width: 34.0,
                height: 14.0,
                color: FURNITURE,
            });
            // Counter top edge
            ctx.draw(&CanvasLine {
                x1: 10.0,
                y1: 29.0,
                x2: 44.0,
                y2: 29.0,
                color: FURNITURE,
            });
            // Espresso machine
            ctx.draw(&CanvasRect {
                x: 14.0,
                y: 29.0,
                width: 8.0,
                height: 7.0,
                color: ACCENT,
            });
            // Steam wand
            ctx.draw(&CanvasLine {
                x1: 18.0,
                y1: 36.0,
                x2: 18.0,
                y2: 39.0,
                color: STRUCTURE_DIM,
            });
            // Cups on counter
            ctx.draw(&CanvasRect {
                x: 26.0,
                y: 29.0,
                width: 3.0,
                height: 3.0,
                color: ACCENT,
            });
            ctx.draw(&CanvasRect {
                x: 31.0,
                y: 29.0,
                width: 3.0,
                height: 3.0,
                color: ACCENT,
            });

            // Bar stools
            for &sx in &[20.0, 30.0, 38.0] {
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
                x: 38.0,
                y: 29.0,
                width: 3.0,
                height: 3.0,
                color: ACCENT,
            });
            ctx.draw(&CanvasLine {
                x1: 39.5,
                y1: 32.0,
                x2: 37.0,
                y2: 38.0,
                color: PLANT,
            });
            ctx.draw(&CanvasLine {
                x1: 39.5,
                y1: 32.0,
                x2: 42.0,
                y2: 37.0,
                color: PLANT,
            });
            ctx.draw(&CanvasLine {
                x1: 39.5,
                y1: 34.0,
                x2: 36.0,
                y2: 40.0,
                color: PLANT,
            });

            // ── Work Tables ─────────────────────────────────────
            draw_table(ctx, 58.0, 15.0);
            draw_table(ctx, 85.0, 15.0);
            draw_table(ctx, 112.0, 15.0);
            draw_table(ctx, 139.0, 15.0);

            // ── Wall Art ────────────────────────────────────────
            // Frame 1
            ctx.draw(&CanvasRect {
                x: 6.0,
                y: 50.0,
                width: 16.0,
                height: 20.0,
                color: ACCENT,
            });
            // Inner mat
            ctx.draw(&CanvasRect {
                x: 8.0,
                y: 52.0,
                width: 12.5,
                height: 16.0,
                color: FURNITURE_DIM,
            });

            // ── Clock ───────────────────────────────────────────
            ctx.draw(&CanvasRect {
                x: 33.0,
                y: 60.0,
                width: 8.0,
                height: 8.0,
                color: ACCENT,
            });
            // Hour hand
            ctx.draw(&CanvasLine {
                x1: 37.0,
                y1: 64.0,
                x2: 37.0,
                y2: 67.0,
                color: LIGHT_GLOW,
            });
            // Minute hand
            ctx.draw(&CanvasLine {
                x1: 37.0,
                y1: 64.0,
                x2: 39.5,
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

            // ── Area Rug (under all seating) ──────────────────
            ctx.draw(&CanvasRect {
                x: 52.0,
                y: 15.5,
                width: 112.0,
                height: 0.5,
                color: ACCENT,
            });
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
