use vn_engine::input::prelude::*;
use vn_engine::prelude::*;
use vn_engine::raylib::prelude::*;
use vn_engine::ui;
use vn_engine::ui::button::ButtonStyle;

use crate::desk::Desk;
use crate::style;

const CAPTION_HEIGHT: f32 = 118.0;

pub struct SearchScreen {
    rooms: Vec<(&'static str, ImageMap)>,
    caption: Option<&'static str>,
    title: TextStyle,
    body: TextStyle,
    panel: PanelStyle,
    back: ButtonStyle,
}

impl SearchScreen {
    pub fn new() -> Self {
        Self {
            rooms: vec![("study", study())],
            caption: None,
            title: style::section(17.0),
            body: style::body(21.0),
            panel: style::frame(PanelStyle::default()),
            back: style::button(ButtonStyle::default().size(220.0, 42.0)),
        }
    }

    fn room(&self, name: &str) -> Option<usize> {
        self.rooms.iter().position(|(id, _)| *id == name)
    }

    fn caption_rect(&self, screen: Vector2) -> Rectangle {
        Rectangle::new(
            40.0,
            screen.y - CAPTION_HEIGHT - 28.0,
            screen.x - 80.0,
            CAPTION_HEIGHT,
        )
    }

    fn back_rect(&self, screen: Vector2) -> Rectangle {
        Rectangle::new(
            screen.x - self.back.width - 40.0,
            28.0,
            self.back.width,
            self.back.height,
        )
    }
}

impl Screen for SearchScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        let screen = ui::screen_size(ctx.rl);
        let area = Rectangle::new(0.0, 0.0, screen.x, screen.y);
        let room = ctx.state.get::<Desk>().room().to_string();
        let Some(index) = self.room(&room) else {
            eprintln!("⚠️ No room to search called '{}'", room);
            return Some(ScreenState::Playing);
        };

        if let Some(picked) = self.rooms[index].1.update(&mut ctx, area) {
            self.caption = Some(spot_text(&room, &picked.id));
            if ctx.state.get_mut::<Desk>().examine(&picked.id) {
                ctx.play_sound("page_turn");
            }
            if room == "study" && picked.id == "desk" {
                ctx.mark_seen(crate::gallery::STUDY_DESK);
            }
            return picked.screen;
        }

        let leaving = [KeyboardKey::KEY_ESCAPE, KeyboardKey::KEY_E]
            .into_iter()
            .any(|key| ctx.rl.is_key_pressed(key))
            || ctx.nav.back;
        let back = self.back_rect(screen);
        (ui::button::button_clicked(&mut ctx, back, &self.back) || leaving)
            .then_some(ScreenState::Playing)
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let screen = ui::screen_size(d);
        let area = Rectangle::new(0.0, 0.0, screen.x, screen.y);
        let fonts = ctx.fonts();
        let Some(index) = self.room(ctx.state.get::<Desk>().room()) else {
            return;
        };
        let map = &self.rooms[index].1;

        map.draw(d, ctx, area);

        let picture = map.frame(ctx.resources, area);
        let desk = ctx.state.get::<Desk>();
        for hotspot in &map.hotspots {
            if desk.examined(&hotspot.id) {
                let mark = hotspot.shape.center(picture);
                d.draw_circle_v(mark, 4.0, style::BRASS.alpha(0.75));
            }
        }

        let caption = self.caption_rect(screen);
        self.panel.draw(d, caption);
        ui::draw_text(
            d,
            fonts,
            "THE STUDY, AS THE HOUSE FOUND IT",
            Vector2::new(caption.x + 26.0, caption.y + 18.0),
            &self.title,
        );
        let text = self
            .caption
            .unwrap_or("The inventory photographs. Look at what was in the room.");
        ui::draw_text_wrapped(
            d,
            fonts,
            text,
            Vector2::new(caption.x + 26.0, caption.y + 48.0),
            caption.width - 52.0,
            &self.body,
        );

        ui::button::draw_button(
            d,
            ctx,
            self.back_rect(screen),
            "Back to the page",
            &self.back,
        );
    }
}

fn study() -> ImageMap {
    let marked = |shape: Shape| shape.in_image(1280.0, 720.0);

    ImageMap::new()
        .background("backgrounds/von_lucis_study.png")
        .style(|s| {
            s.hovered(
                Highlight::new()
                    .fill(style::BRASS.alpha(0.12))
                    .border(1.5, style::BRASS),
            )
            .focused(Highlight::new().border(1.5, style::PARCHMENT))
            .label(
                LabelStyle::default()
                    .at(LabelAt::Pointer)
                    .text(TextStyle::new(
                        vn_engine::ui::fonts::FontRole::Menu,
                        17.0,
                        style::PARCHMENT,
                    ))
                    .panel(Some(style::plate(PanelStyle::default()))),
            )
            .hover_sound("page_turn")
        })
        .hotspot(
            Hotspot::new(
                "desk",
                marked(Shape::polygon([
                    (0.0, 332.0),
                    (415.0, 344.0),
                    (420.0, 470.0),
                    (180.0, 620.0),
                    (0.0, 604.0),
                ])),
            )
            .label("The writing desk")
            .tooltip("Where the letters were read, and burned"),
        )
        .hotspot(
            Hotspot::new("lamp", marked(Shape::circle(212.0, 258.0, 46.0))).label("The oil lamp"),
        )
        .hotspot(
            Hotspot::new("window", marked(Shape::rect(300.0, 0.0, 176.0, 330.0)))
                .label("The tall window"),
        )
        .hotspot(
            Hotspot::new("chair", marked(Shape::rect(700.0, 258.0, 240.0, 246.0)))
                .label("The armchair"),
        )
        .hotspot(
            Hotspot::new("portrait", marked(Shape::rect(896.0, 24.0, 68.0, 162.0)))
                .label("Two small portraits"),
        )
        .hotspot(
            Hotspot::new(
                "fireplace",
                marked(Shape::rect(1016.0, 330.0, 250.0, 176.0)),
            )
            .label("The fireplace")
            .tooltip("Three envelopes went in here"),
        )
}

fn spot_text(room: &str, spot: &str) -> &'static str {
    match (room, spot) {
        ("study", "desk") => {
            "The blotter kept the ghost of a signature and three words pressed backwards into it: \
             Von Lucis. Three days."
        }
        ("study", "lamp") => {
            "The lamp was found lit, in a house nobody had entered for a week. \
             The oil was full."
        }
        ("study", "window") => {
            "Mary wrote that she looked at the windows first. \
             \"Outside it is not night. Outside it is not anything.\""
        }
        ("study", "chair") => {
            "The chair faces the door, not the fire. He sat in it waiting for somebody \
             who did not use doors."
        }
        ("study", "portrait") => {
            "Sofia Von Lucis, 1889, and a second frame the House catalogued as empty. \
             It is not empty. It is a photograph of the same room."
        }
        ("study", "fireplace") => {
            "Ash to the grate's lip, and one corner of pale paper that did not burn. \
             It still smells of perfume."
        }
        _ => "The photographs show nothing here.",
    }
}
