use vn_engine::raylib::prelude::*;
use vn_engine::ui::{self, ButtonStyle};
use vn_engine::{
    DragBoard, Draggable, DrawContext, DropTarget, GameContext, Highlight, LabelAt, LabelStyle,
    PanelStyle, Screen, ScreenState, Shape, TextStyle,
};

use crate::desk::{self, BOX, Desk, REPORT};
use crate::evidence::{Evidence, describe};
use crate::style;

const CARD: (f32, f32) = (0.15, 0.30);
const TRAY_CARD: (f32, f32) = (0.10, 0.20);

pub struct ReportDesk {
    board: DragBoard,
    heading: TextStyle,
    hint: TextStyle,
    card_text: TextStyle,
    tray_text: TextStyle,
    panel: PanelStyle,
    card: PanelStyle,
    done: ButtonStyle,
}

impl ReportDesk {
    pub fn new() -> Self {
        Self {
            board: board(),
            heading: style::heading(34.0),
            hint: style::label(17.0),
            card_text: style::body(16.0),
            tray_text: style::section(18.0),
            panel: style::frame(PanelStyle::default()),
            card: style::plate(PanelStyle::default()),
            done: style::button(ButtonStyle::default().size(220.0, 44.0)),
        }
    }

    fn panel_rect(&self, screen: Vector2) -> Rectangle {
        Rectangle::new(60.0, 24.0, screen.x - 120.0, screen.y - 48.0)
    }

    fn board_rect(&self, screen: Vector2) -> Rectangle {
        let panel = self.panel_rect(screen);
        Rectangle::new(
            panel.x + 40.0,
            panel.y + 96.0,
            panel.width - 80.0,
            panel.height - 180.0,
        )
    }

    fn done_rect(&self, screen: Vector2) -> Rectangle {
        let panel = self.panel_rect(screen);
        Rectangle::new(
            panel.x + (panel.width - self.done.width) / 2.0,
            panel.y + panel.height - self.done.height - 24.0,
            self.done.width,
            self.done.height,
        )
    }

    fn cards(&self, evidence: &Evidence, desk: &Desk) -> Vec<Draggable> {
        let mut counts = [0usize; 3];
        evidence
            .items()
            .map(|(item, _)| {
                let tray = desk.tray(item);
                let column = match tray {
                    REPORT => 1,
                    BOX => 2,
                    _ => 0,
                };
                let index = counts[column];
                counts[column] += 1;
                Draggable::new(item, card_shape(tray, index)).label(describe(item).1)
            })
            .collect()
    }
}

impl Screen for ReportDesk {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(
            &mut ctx,
            Some(&style::background(style::ARCHIVE_BACKGROUND)),
        );
        let screen = ui::screen_size(ctx.rl);
        let cards = self.cards(ctx.state.get::<Evidence>(), ctx.state.get::<Desk>());
        self.board.set_items(cards);

        if let Some(drop) = self.board.update(&mut ctx, self.board_rect(screen)) {
            match (drop.accepted, drop.target_id.as_deref()) {
                (true, Some(tray)) => {
                    let tray = tray.to_string();
                    ctx.state.get_mut::<Desk>().file(&drop.item_id, &tray);
                    ctx.notify(format!(
                        "{}: {}",
                        desk::tray_name(&tray),
                        describe(&drop.item_id).0
                    ));
                }
                (false, Some(_)) => ctx.notify_error("Box fourteen is the box. It stays."),
                _ => {}
            }
            return None;
        }

        if self.board.held().is_some() {
            return None;
        }

        let done = ui::button_clicked(&mut ctx, self.done_rect(screen), &self.done);
        let key = [KeyboardKey::KEY_ESCAPE, KeyboardKey::KEY_BACKSPACE]
            .into_iter()
            .any(|key| ctx.rl.is_key_pressed(key));
        (done || key).then_some(ScreenState::Playing)
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();
        let panel = self.panel_rect(screen);
        let area = self.board_rect(screen);

        ui::draw_background(
            d,
            ctx.resources,
            Some(&style::background(style::ARCHIVE_BACKGROUND)),
        );
        d.draw_rectangle(0, 0, screen.x as i32, screen.y as i32, style::BACKDROP);
        self.panel.draw(d, panel);

        ui::draw_text_centered(
            d,
            fonts,
            "What goes in report 222",
            Vector2::new(screen.x / 2.0, panel.y + 48.0),
            &self.heading,
        );
        ui::draw_text_centered(
            d,
            fonts,
            "Drag what you have filed into a tray. Arrow keys and Enter do the same.",
            Vector2::new(screen.x / 2.0, panel.y + 78.0),
            &self.hint,
        );

        self.board.draw(d, ctx, area);

        for (index, target) in self.board.targets.iter().enumerate() {
            let bounds = target.shape.bounds(area);
            let name = desk::tray_name(&self.board.targets[index].id);
            ui::draw_text_centered(
                d,
                fonts,
                name,
                Vector2::new(bounds.x + bounds.width / 2.0, bounds.y - 16.0),
                &self.tray_text,
            );
        }

        for (index, item) in self.board.items.iter().enumerate() {
            let rect = self.board.item_rect(index, area);
            self.card.draw(d, rect);
            let name = describe(&item.id).0;
            let text = ui::fit_text(fonts, &self.card_text, name, rect.width - 16.0);
            ui::draw_text_centered(
                d,
                fonts,
                &text,
                Vector2::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0),
                &self.card_text,
            );
        }

        ui::draw_button(d, ctx, self.done_rect(screen), "Done", &self.done);
    }
}

fn board() -> DragBoard {
    DragBoard::new()
        .style(|s| {
            s.item(Highlight::new().fill(style::RAISED.alpha(0.0)))
                .item_hovered(Highlight::new().border(1.5, style::BRASS))
                .item_held(Highlight::new().opacity(0.9).border(1.5, style::PARCHMENT))
                .target(Highlight::new().border(1.0, style::BRASS_DIM.alpha(0.5)))
                .target_ready(Highlight::new().border(1.5, style::BRASS))
                .target_blocked(Highlight::new().border(1.5, style::OXBLOOD_HOVER))
                .target_hovered(Highlight::new().fill(style::BRASS.alpha(0.12)))
                .label(
                    LabelStyle::default()
                        .at(LabelAt::Below)
                        .text(style::label(15.0))
                        .panel(Some(style::plate(PanelStyle::default()))),
                )
                .sounds("page_turn", "book_close", "page_turn")
        })
        .target(
            DropTarget::new(REPORT, Shape::rect(0.04, 0.52, 0.44, 0.44))
                .label("Everything here is read by the House")
                .accepts(|item, _| item != "box_14"),
        )
        .target(
            DropTarget::new(BOX, Shape::rect(0.52, 0.52, 0.44, 0.44))
                .label("Back on the shelf, uncatalogued"),
        )
}

fn card_shape(tray: &str, index: usize) -> Shape {
    let index = index as f32;
    match tray {
        REPORT => Shape::rect(0.06 + index * 0.105, 0.62, TRAY_CARD.0, TRAY_CARD.1),
        BOX => Shape::rect(0.54 + index * 0.105, 0.62, TRAY_CARD.0, TRAY_CARD.1),
        _ => Shape::rect(0.03 + index * 0.165, 0.04, CARD.0, CARD.1),
    }
}
