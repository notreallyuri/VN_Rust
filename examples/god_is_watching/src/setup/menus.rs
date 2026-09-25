use novn::prelude::*;
use novn::screens::prelude::*;
use novn::ui::layout::Anchor;

use super::{CREDITS, GALLERY, SUBTITLE, screen};
use crate::journal::CaseLedger;
use crate::style;

fn link(label: &'static str, action: Action) -> MenuItem {
    MenuItem::new(label, action).style(move |b| b.size(style::link_width(label), 36.0))
}

pub fn menus(app: VnApp) -> VnApp {
    app.start_screen(|s| {
        s.background(style::background(style::TITLE_BACKGROUND))
            .scenery(|s| style::scenery(s).letterbox(|l| l.slide_in(1.6)))
            .title("GOD IS WATCHING")
            .title_text(style::title_card())
            .title_y(style::TITLE_CARD_Y)
            .subtitle(SUBTITLE)
            .subtitle_text(style::title_card_subtitle())
            .prompt("PRESS ANY KEY")
            .prompt_text(style::section(14.0).spacing(6.0))
            .prompt_in_bar(true)
            .prompt_pulse(2.6)
            .footer("v0.3 · made with novn")
            .footer_text(style::label(12.0))
    })
    .main_menu(|m| {
        m.background(style::background(style::TITLE_BACKGROUND))
            .scenery(style::scenery)
            .intro(1.4)
            .title("GOD IS WATCHING")
            .title_text(style::title_card())
            .title_y(style::TITLE_CARD_Y)
            .subtitle(SUBTITLE)
            .subtitle_text(style::title_card_subtitle())
            .buttons_in_bar(true)
            .button_style(|b| style::menu_link(b.size(132.0, 36.0)))
            .layout(|l| l.row().anchor(Anchor::Center).spacing(34.0))
            .separator(7.0, |p| p.color(style::BRASS_DIM))
            .item(link("NEW GAME", Action::NewGame))
            .item(
                link("CONTINUE", Action::Continue)
                    .tooltip("Back to where you left off, or your most recent save"),
            )
            .item(link("LOAD", Action::Goto(ScreenState::Load)))
            .item(link("SETTINGS", Action::Goto(ScreenState::Settings)))
            .item(link("GALLERY", Action::Goto(screen(GALLERY))))
            .item(
                link("CREDITS", Action::Goto(screen(CREDITS)))
                    .enabled_if(|view| view.persistent.get::<CaseLedger>().closed() > 0)
                    .tooltip("Close the case once to open the credits"),
            )
            .item(
                MenuItem::new("EXIT", Action::confirm("Leave the Archive?", Action::Quit))
                    .style(|b| style::menu_link_danger(b.size(style::link_width("EXIT"), 36.0))),
            )
    })
    .pause_menu(|p| {
        p.title_text(style::heading(38.0))
            .button_style(|b| b.size(230.0, 44.0))
            .layout(|l| l.grid(2).spacing_xy(12.0, 10.0))
    })
    .confirm_dialog(|c| c.message_text(style::body(22.0)))
}
