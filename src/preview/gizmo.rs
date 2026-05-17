use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::Clear;
use ratatui::widgets::canvas::{Canvas, Line as CanvasLine};

const X_COLOR: Color = Color::Red;
const Y_COLOR: Color = Color::Green;
const Z_COLOR: Color = Color::Blue;

pub fn render(frame: &mut Frame<'_>, area: Rect, rotation: [f32; 3]) {
    if area.width < 4 || area.height < 3 {
        return;
    }

    let rot = rotation_matrix(rotation[0], rotation[1], rotation[2]);

    // OpenSCAD axes mapped into Bevy world space (matches the (x,y,z) → (x,z,-y)
    // swap that `stl_to_obj` applies to mesh data).
    let x_world = apply(&rot, [1.0, 0.0, 0.0]);
    let y_world = apply(&rot, [0.0, 0.0, -1.0]);
    let z_world = apply(&rot, [0.0, 1.0, 0.0]);

    // Orthographic projection: keep (x, y), discard z.
    let x = [f64::from(x_world[0]), f64::from(x_world[1])];
    let y = [f64::from(y_world[0]), f64::from(y_world[1])];
    let z = [f64::from(z_world[0]), f64::from(z_world[1])];

    frame.render_widget(Clear, area);
    let canvas = Canvas::default()
        .x_bounds([-1.4, 1.4])
        .y_bounds([-1.4, 1.4])
        .marker(symbols::Marker::Braille)
        .paint(move |ctx| {
            ctx.draw(&CanvasLine {
                x1: 0.0,
                y1: 0.0,
                x2: x[0],
                y2: x[1],
                color: X_COLOR,
            });
            ctx.draw(&CanvasLine {
                x1: 0.0,
                y1: 0.0,
                x2: y[0],
                y2: y[1],
                color: Y_COLOR,
            });
            ctx.draw(&CanvasLine {
                x1: 0.0,
                y1: 0.0,
                x2: z[0],
                y2: z[1],
                color: Z_COLOR,
            });
            ctx.print(x[0], x[1], label("X", X_COLOR));
            ctx.print(y[0], y[1], label("Y", Y_COLOR));
            ctx.print(z[0], z[1], label("Z", Z_COLOR));
        });
    frame.render_widget(canvas, area);
}

fn label(text: &'static str, color: Color) -> Line<'static> {
    Line::styled(
        text,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

fn rotation_matrix(rx_deg: f32, ry_deg: f32, rz_deg: f32) -> [[f32; 3]; 3] {
    let rx = rx_deg.to_radians();
    let ry = ry_deg.to_radians();
    let rz = rz_deg.to_radians();
    let (sx, cx) = (rx.sin(), rx.cos());
    let (sy, cy) = (ry.sin(), ry.cos());
    let (sz, cz) = (rz.sin(), rz.cos());

    // Bevy EulerRot::XYZ intrinsic → combined matrix R = Rx · Ry · Rz.
    [
        [cy * cz, -cy * sz, sy],
        [cx * sz + sx * sy * cz, cx * cz - sx * sy * sz, -sx * cy],
        [sx * sz - cx * sy * cz, sx * cz + cx * sy * sz, cx * cy],
    ]
}

fn apply(m: &[[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}
