use std::rc::Rc;

use novn_script::Position;
use raylib::prelude::*;

use super::{ChoiceImageStyle, ChoicePreviewStyle, DialogueBoxStyle, NvlStyle, PlayingKeys};
use crate::action::Action;
use crate::data::resources::{choice_path, preview_path};
use crate::screen::ScreenState;
use crate::screens::log::LOG_OVERLAY;
use crate::screens::pause_menu::PAUSE_OVERLAY;
use crate::ui::button::ButtonStyle;
use crate::ui::button::StyleOverride;
use crate::ui::fonts::FontRole;
use crate::ui::layout::{Anchor, Layout};
use crate::ui::shape::PanelStyle;
use crate::ui::styled::StyledText;
use crate::ui::theme::Theme;
use crate::ui::{Background, TextStyle};

#[derive(Clone, Debug)]
pub struct HudButton {
    pub label: String,
    pub action: Action,
    pub tooltip: Option<String>,
    pub style: Option<StyleOverride>,
    pub group: Option<String>,
}

type ChoiceStyleFn = Rc<dyn Fn(usize, &str, ButtonStyle) -> ButtonStyle>;

#[derive(Clone)]
pub struct ChoiceStyle(ChoiceStyleFn);

impl std::fmt::Debug for ChoiceStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChoiceStyle(..)")
    }
}

impl HudButton {
    pub fn new(label: impl Into<String>, action: Action) -> Self {
        Self {
            label: label.into(),
            action,
            tooltip: None,
            style: None,
            group: None,
        }
    }

    pub fn group(mut self, name: impl Into<String>) -> Self {
        self.group = Some(name.into());
        self
    }

    pub fn style(mut self, style: impl Fn(ButtonStyle) -> ButtonStyle + 'static) -> Self {
        self.style = Some(StyleOverride::new(style));
        self
    }

    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }
}

#[derive(Clone, Debug)]
pub struct PlayingConfig {
    pub dialogue_box: DialogueBoxStyle,
    pub speaker_text: TextStyle,
    pub dialogue_text: TextStyle,
    pub choice_button: ButtonStyle,
    pub choice_style: Option<ChoiceStyle>,
    pub choice_layout: Layout,
    pub choice_image: ChoiceImageStyle,
    pub choice_preview: ChoicePreviewStyle,
    pub nvl: NvlStyle,
    pub end_title: String,
    pub end_title_text: TextStyle,
    pub end_hint: String,
    pub end_hint_text: TextStyle,
    pub advance_keys: Vec<KeyboardKey>,
    pub menu_key: Option<KeyboardKey>,
    pub after_end: ScreenState,
    pub background: Option<Background>,
    pub positions: [f32; 5],
    pub character_height: Option<f32>,
    pub hud: Vec<HudButton>,
    custom_hud: bool,
    pub keys: PlayingKeys,
    pub skip_interval: f64,
    pub auto_per_character: f64,
    pub indicator_text: TextStyle,
    pub indicator: PanelStyle,
    pub skip_label: String,
    pub auto_label: String,
    pub log_overlay: String,
    pub hud_button: ButtonStyle,
    pub hud_margin: f32,
    pub hud_layout: Layout,
    pub hud_groups: Vec<(String, Layout)>,
    pub quick_save_key: Option<KeyboardKey>,
    pub quick_load_key: Option<KeyboardKey>,
    pub pause_key: Option<KeyboardKey>,
    pub pause_overlay: String,
}

impl Default for PlayingConfig {
    fn default() -> Self {
        Self {
            dialogue_box: DialogueBoxStyle::default(),
            speaker_text: TextStyle::new(FontRole::Speaker, 24.0, Color::GOLD),
            dialogue_text: TextStyle::new(FontRole::Dialogue, 26.0, Color::RAYWHITE),
            choice_button: ButtonStyle::default()
                .size(720.0, 56.0)
                .color(Color::new(30, 30, 45, 230))
                .font(FontRole::Choice),
            choice_style: None,
            choice_layout: Layout::default().spacing(16.0),
            choice_image: ChoiceImageStyle::default(),
            choice_preview: ChoicePreviewStyle::default(),
            nvl: NvlStyle::default(),
            end_title: "The End".to_string(),
            end_title_text: TextStyle::new(FontRole::Title, 56.0, Color::RAYWHITE),
            end_hint: "Click to return to the menu".to_string(),
            end_hint_text: TextStyle::new(FontRole::Menu, 20.0, Color::GRAY),
            advance_keys: vec![KeyboardKey::KEY_SPACE, KeyboardKey::KEY_ENTER],
            menu_key: None,
            after_end: ScreenState::MainMenu,
            background: None,
            positions: [0.15, 0.3, 0.5, 0.7, 0.85],
            character_height: Some(0.8),
            hud: vec![
                HudButton::new("Log", Action::overlay(LOG_OVERLAY))
                    .tooltip("Everything so far (L)"),
                HudButton::new("Auto", Action::ToggleAuto)
                    .tooltip("Advance on its own after each line (A)"),
                HudButton::new("Skip", Action::ToggleSkip)
                    .tooltip("Skip lines you've already seen (Tab, or hold Ctrl)"),
            ],
            custom_hud: false,
            keys: PlayingKeys::default(),
            skip_interval: 0.05,
            auto_per_character: 0.02,
            indicator_text: TextStyle::new(FontRole::Menu, 18.0, Color::RAYWHITE),
            indicator: PanelStyle::new(Color::new(0, 0, 0, 160)).roundness(0.3),
            skip_label: "Skip »".to_string(),
            auto_label: "Auto".to_string(),
            log_overlay: LOG_OVERLAY.to_string(),
            hud_button: ButtonStyle::default()
                .size(130.0, 40.0)
                .color(Color::new(20, 20, 30, 190))
                .font_size(18.0),
            hud_margin: 16.0,
            hud_layout: Layout::default()
                .row()
                .anchor(Anchor::TopRight)
                .spacing(10.0),
            hud_groups: Vec::new(),
            quick_save_key: Some(KeyboardKey::KEY_F5),
            quick_load_key: Some(KeyboardKey::KEY_F9),
            pause_key: Some(KeyboardKey::KEY_ESCAPE),
            pause_overlay: PAUSE_OVERLAY.to_string(),
        }
    }
}

impl PlayingConfig {
    pub fn dialogue_box(
        mut self,
        style: impl FnOnce(DialogueBoxStyle) -> DialogueBoxStyle,
    ) -> Self {
        self.dialogue_box = style(self.dialogue_box);
        self
    }

    pub fn speaker_text(mut self, style: TextStyle) -> Self {
        self.speaker_text = style;
        self
    }

    pub fn dialogue_text(mut self, style: TextStyle) -> Self {
        self.dialogue_text = style;
        self
    }

    pub fn choice_button(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.choice_button = style(self.choice_button);
        self
    }

    pub fn choice_button_for(
        mut self,
        style: impl Fn(usize, &str, ButtonStyle) -> ButtonStyle + 'static,
    ) -> Self {
        self.choice_style = Some(ChoiceStyle(Rc::new(style)));
        self
    }

    pub fn choice_style_for(&self, index: usize, text: &str) -> ButtonStyle {
        match &self.choice_style {
            Some(ChoiceStyle(style)) => style(index, text, self.choice_button.clone()),
            None => self.choice_button.clone(),
        }
    }

    pub fn choice_image(
        mut self,
        style: impl FnOnce(ChoiceImageStyle) -> ChoiceImageStyle,
    ) -> Self {
        self.choice_image = style(self.choice_image);
        self
    }

    pub fn choice_preview(
        mut self,
        style: impl FnOnce(ChoicePreviewStyle) -> ChoicePreviewStyle,
    ) -> Self {
        self.choice_preview = style(self.choice_preview);
        self
    }

    pub fn nvl(mut self, style: impl FnOnce(NvlStyle) -> NvlStyle) -> Self {
        self.nvl = style(self.nvl);
        self
    }

    pub fn option_style(&self, option: &novn_script::ChoiceOption) -> ButtonStyle {
        let style = self.choice_style_for(option.index, &option.text);
        match &option.image {
            Some(image) => self.choice_image.apply(style, choice_path(image)),
            None => style,
        }
    }

    pub fn option_pictures(&self, options: &[novn_script::ChoiceOption]) -> Vec<String> {
        options
            .iter()
            .flat_map(|option| {
                option
                    .image
                    .as_deref()
                    .map(choice_path)
                    .into_iter()
                    .chain(option.preview.as_deref().map(preview_path))
            })
            .collect()
    }

    pub fn hud_style(&self, index: usize) -> ButtonStyle {
        match self.hud.get(index).and_then(|button| button.style.as_ref()) {
            Some(style) => style.apply(&self.hud_button),
            None => self.hud_button.clone(),
        }
    }

    pub fn choice_spacing(mut self, spacing: f32) -> Self {
        self.choice_layout = self.choice_layout.spacing(spacing);
        self
    }

    pub fn choice_layout(mut self, layout: impl FnOnce(Layout) -> Layout) -> Self {
        self.choice_layout = layout(self.choice_layout);
        self
    }

    pub fn end_title(mut self, text: impl Into<String>) -> Self {
        self.end_title = text.into();
        self
    }

    pub fn end_title_text(mut self, style: TextStyle) -> Self {
        self.end_title_text = style;
        self
    }

    pub fn end_hint(mut self, text: impl Into<String>) -> Self {
        self.end_hint = text.into();
        self
    }

    pub fn end_hint_text(mut self, style: TextStyle) -> Self {
        self.end_hint_text = style;
        self
    }

    pub fn advance_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.advance_keys = keys.into_iter().collect();
        self
    }

    pub fn menu_key(mut self, key: Option<KeyboardKey>) -> Self {
        self.menu_key = key;
        self
    }

    pub fn after_end(mut self, state: ScreenState) -> Self {
        self.after_end = state;
        self
    }

    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }

    pub fn position(mut self, position: Position, x: f32) -> Self {
        self.positions[position_index(position)] = x;
        self
    }

    pub fn position_x(&self, position: Position) -> f32 {
        self.positions[position_index(position)]
    }

    pub fn character_height(mut self, fraction: Option<f32>) -> Self {
        self.character_height = fraction;
        self
    }

    pub fn hud_button(self, label: impl Into<String>, action: Action) -> Self {
        self.hud_item(HudButton::new(label, action))
    }

    pub fn hud_item(mut self, button: HudButton) -> Self {
        if !self.custom_hud {
            self.hud.clear();
            self.custom_hud = true;
        }
        self.hud.push(button);
        self
    }

    pub fn hud_button_style(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.hud_button = style(self.hud_button);
        self
    }

    pub fn hud_margin(mut self, margin: f32) -> Self {
        self.hud_margin = margin;
        self
    }

    pub fn hud_spacing(mut self, spacing: f32) -> Self {
        self.hud_layout = self.hud_layout.spacing(spacing);
        self
    }

    pub fn hud_layout(mut self, layout: impl FnOnce(Layout) -> Layout) -> Self {
        self.hud_layout = layout(self.hud_layout);
        self
    }

    pub fn quick_save_key(mut self, key: Option<KeyboardKey>) -> Self {
        self.quick_save_key = key;
        self
    }

    pub fn quick_load_key(mut self, key: Option<KeyboardKey>) -> Self {
        self.quick_load_key = key;
        self
    }

    pub fn pause_key(mut self, key: Option<KeyboardKey>) -> Self {
        self.pause_key = key;
        self
    }

    pub fn keys(mut self, keys: impl FnOnce(PlayingKeys) -> PlayingKeys) -> Self {
        self.keys = keys(self.keys);
        self
    }

    pub fn skip_interval(mut self, seconds: f64) -> Self {
        self.skip_interval = seconds.max(0.0);
        self
    }

    pub fn auto_per_character(mut self, seconds: f64) -> Self {
        self.auto_per_character = seconds.max(0.0);
        self
    }

    pub fn indicator_text(mut self, style: TextStyle) -> Self {
        self.indicator_text = style;
        self
    }

    pub fn indicator_color(mut self, color: Color) -> Self {
        self.indicator.color = color;
        self
    }

    pub fn indicator(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.indicator = style(self.indicator);
        self
    }

    pub fn skip_label(mut self, text: impl Into<String>) -> Self {
        self.skip_label = text.into();
        self
    }

    pub fn auto_label(mut self, text: impl Into<String>) -> Self {
        self.auto_label = text.into();
        self
    }

    pub fn log_overlay(mut self, name: impl Into<String>) -> Self {
        self.log_overlay = name.into();
        self
    }

    pub fn auto_delay(&self, settings: &crate::data::settings::Settings, text: &str) -> f64 {
        settings.auto_seconds() + StyledText::parse(text).chars() as f64 * self.auto_per_character
    }

    pub fn pause_overlay(mut self, name: impl Into<String>) -> Self {
        self.pause_overlay = name.into();
        self
    }

    pub fn hud_group(
        mut self,
        name: impl Into<String>,
        layout: impl FnOnce(Layout) -> Layout,
    ) -> Self {
        let name = name.into();
        let base = self.hud_layout.clone();
        match self.hud_groups.iter_mut().find(|(n, _)| *n == name) {
            Some((_, existing)) => *existing = layout(existing.clone()),
            None => self.hud_groups.push((name, layout(base))),
        }
        self
    }

    fn group_layout(&self, group: Option<&str>) -> &Layout {
        group
            .and_then(|name| self.hud_groups.iter().find(|(n, _)| n == name))
            .map_or(&self.hud_layout, |(_, layout)| layout)
    }

    pub fn hud_rects(&self, screen: Vector2) -> Vec<Rectangle> {
        let area = inset(screen, self.hud_margin);
        let mut rects = vec![Rectangle::new(0.0, 0.0, 0.0, 0.0); self.hud.len()];
        let mut groups: Vec<Option<&str>> = Vec::new();
        for button in &self.hud {
            if !groups.contains(&button.group.as_deref()) {
                groups.push(button.group.as_deref());
            }
        }
        for group in groups {
            let members: Vec<usize> = (0..self.hud.len())
                .filter(|&i| self.hud[i].group.as_deref() == group)
                .collect();
            let sizes: Vec<Vector2> = members
                .iter()
                .map(|&i| {
                    let style = self.hud_style(i);
                    Vector2::new(style.width, style.height)
                })
                .collect();
            let placed = self.group_layout(group).place(area, &sizes);
            for (i, rect) in members.into_iter().zip(placed) {
                rects[i] = rect;
            }
        }
        rects
    }

    pub fn choice_rects(&self, count: usize, screen: Vector2) -> Vec<Rectangle> {
        let button = &self.choice_button;
        let area = inset(screen, self.dialogue_box.margin);
        let scale = crate::ui::reading::scale();
        let size = Vector2::new(
            (button.width * scale).min(area.width),
            button.height * scale,
        );
        self.choice_layout.place(area, &vec![size; count])
    }
}

fn position_index(position: Position) -> usize {
    Position::ALL
        .iter()
        .position(|p| *p == position)
        .expect("every position is in Position::ALL")
}

fn inset(screen: Vector2, margin: f32) -> Rectangle {
    Rectangle::new(
        margin,
        margin,
        (screen.x - margin * 2.0).max(0.0),
        (screen.y - margin * 2.0).max(0.0),
    )
}

impl PlayingConfig {
    pub fn themed(mut self, theme: &Theme) -> Self {
        self.dialogue_text = theme.text(&theme.body, self.dialogue_text);
        self.indicator_text = theme.text(&theme.section, self.indicator_text);
        self.end_title_text = theme.text(&theme.title, self.end_title_text);
        self.end_hint_text = theme.text(&theme.label, self.end_hint_text);
        self.choice_button = theme.buttons(&theme.button, self.choice_button);
        self.hud_button = theme.buttons(&theme.button, self.hud_button);
        self.dialogue_box.panel = theme.surface(&theme.panel, self.dialogue_box.panel);
        self.choice_preview.panel = theme.surface(&theme.panel, self.choice_preview.panel);
        self.nvl.panel = theme.surface(&theme.panel, self.nvl.panel);
        self.indicator = theme.surface(&theme.plate, self.indicator);
        if let Some(plate) = self.dialogue_box.name_plate.as_mut() {
            plate.panel = theme.surface(&theme.plate, plate.panel);
        }
        self
    }
}
