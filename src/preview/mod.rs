mod camera;
mod gizmo;

const GIZMO_WIDTH: u16 = 10;
const GIZMO_HEIGHT: u16 = 5;

use std::io;

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::Frame;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};
use ratatui_ratty::{ObjectFormat, RattyGraphic, RattyGraphicSettings};

use crate::status::{desc_style, key_style};

const PREVIEW_ID: u32 = 1;
const DIMMED_BRIGHTNESS: f32 = 0.35;
const ACTIVE_BRIGHTNESS: f32 = 1.0;

pub struct PreviewPane {
    graphic: RattyGraphic<'static>,
    has_mesh: bool,
    dragging: Option<Position>,
}

impl PreviewPane {
    pub fn new() -> Self {
        // Isometric: yaw 45° around Y, pitch arctan(1/√2) ≈ 35.264° around X.
        // Sign of rx is negative so the camera "looks down" at the +Z (OpenSCAD up) face.
        let graphic = RattyGraphic::new(
            RattyGraphicSettings::new("ratscad-live.obj")
                .id(PREVIEW_ID)
                .format(ObjectFormat::Obj)
                .animate(false)
                .scale(0.5)
                .brightness(ACTIVE_BRIGHTNESS)
                .color([255, 217, 112])
                .rotation([33.0, -125.0, 0.0]),
        );
        Self {
            graphic,
            has_mesh: false,
            dragging: None,
        }
    }

    pub fn register_mesh(&mut self, obj_bytes: &[u8]) -> io::Result<()> {
        self.graphic.settings_mut().brightness = ACTIVE_BRIGHTNESS;
        self.graphic.register_payload(obj_bytes)?;
        self.has_mesh = true;
        Ok(())
    }

    pub fn rotate(&mut self, dy: i8, dx: i8) -> io::Result<()> {
        const STEP_DEG: f32 = 5.0;
        self.graphic.settings_mut().rotation[0] += f32::from(dy) * STEP_DEG;
        self.graphic.settings_mut().rotation[1] += f32::from(dx) * STEP_DEG;
        if self.has_mesh {
            self.graphic.update()?;
        }
        Ok(())
    }

    pub fn zoom(&mut self, direction: i8) -> io::Result<()> {
        let next = camera::scroll_to_scale(self.graphic.settings().scale, direction);
        self.graphic.settings_mut().scale = next;
        if self.has_mesh {
            self.graphic.update()?;
        }
        Ok(())
    }

    pub fn pan(&mut self, dx: i8, dy: i8) -> io::Result<()> {
        const STEP_PX: f32 = 20.0;
        self.graphic.settings_mut().offset[0] += f32::from(dx) * STEP_PX;
        self.graphic.settings_mut().offset[1] += f32::from(dy) * STEP_PX;
        if self.has_mesh {
            self.graphic.update()?;
        }
        Ok(())
    }

    pub fn set_dim(&mut self, dim: bool) -> io::Result<()> {
        let target = if dim { DIMMED_BRIGHTNESS } else { ACTIVE_BRIGHTNESS };
        if (self.graphic.settings().brightness - target).abs() > f32::EPSILON {
            self.graphic.settings_mut().brightness = target;
            if self.has_mesh {
                self.graphic.update()?;
            }
        }
        Ok(())
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.graphic.clear()?;
        self.has_mesh = false;
        Ok(())
    }

    pub fn on_mouse(&mut self, event: MouseEvent) -> io::Result<()> {
        let pos = Position::new(event.column, event.row);
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.dragging = Some(pos);
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if let Some(prev) = self.dragging {
                    let dx = pos.x as i16 - prev.x as i16;
                    let dy = pos.y as i16 - prev.y as i16;
                    let [drx, dry] = camera::drag_to_rotation(dx, dy);
                    self.graphic.settings_mut().rotation[0] += drx;
                    self.graphic.settings_mut().rotation[1] += dry;
                    self.dragging = Some(pos);
                    if self.has_mesh {
                        self.graphic.update()?;
                    }
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.dragging = None;
            }
            MouseEventKind::ScrollUp => {
                let next = camera::scroll_to_scale(self.graphic.settings().scale, 1);
                self.graphic.settings_mut().scale = next;
                if self.has_mesh {
                    self.graphic.update()?;
                }
            }
            MouseEventKind::ScrollDown => {
                let next = camera::scroll_to_scale(self.graphic.settings().scale, -1);
                self.graphic.settings_mut().scale = next;
                if self.has_mesh {
                    self.graphic.update()?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn render_toolbar(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        border: bool,
        show_keys: bool,
    ) {
        let content_area = if border {
            let block = Block::default()
                .borders(Borders::LEFT)
                .border_style(Style::default().fg(Color::DarkGray));
            let inner = block.inner(area);
            block.render(area, frame.buffer_mut());
            inner
        } else {
            area
        };

        if !show_keys {
            return;
        }
        let key = key_style();
        let desc = desc_style();
        let spans = vec![
            Span::raw(" "),
            Span::styled(" arrows ", key),
            Span::styled(" rotate ", desc),
            Span::styled(" Ctrl+arr ", key),
            Span::styled(" pan ", desc),
            Span::styled(" z/x ", key),
            Span::styled(" zoom ", desc),
            Span::styled(" f ", key),
            Span::styled(" fullscreen", desc),
        ];
        frame.render_widget(Paragraph::new(Line::from(spans)), content_area);
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(Color::DarkGray));
        let inner = block.inner(area);
        block.render(area, frame.buffer_mut());
        frame.render_widget(Clear, inner);

        if self.has_mesh {
            (&self.graphic).render(inner, frame.buffer_mut());
        }

        let gw = GIZMO_WIDTH.min(inner.width);
        let gh = GIZMO_HEIGHT.min(inner.height);
        if gw >= 4 && gh >= 3 {
            let gizmo_area = Rect {
                x: inner.x,
                y: inner.y + inner.height - gh,
                width: gw,
                height: gh,
            };
            gizmo::render(frame, gizmo_area, self.graphic.settings().rotation);
        }
    }
}
