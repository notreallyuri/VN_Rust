use novn_script::Event;
use novn_script::markup::plain;

use super::PlayingScreen;
use crate::context::GameContext;
use crate::game::reader::Reading;

impl PlayingScreen {
    pub(super) fn toggle_voicing(&mut self, ctx: &mut GameContext) {
        if ctx.settings.values.self_voicing {
            ctx.notify("Self-voicing off");
            ctx.settings.update(|s| s.self_voicing = false);
        } else {
            ctx.settings.update(|s| s.self_voicing = true);
            ctx.notify("Self-voicing on");
        }
    }

    pub(super) fn voice(&mut self, ctx: &GameContext) {
        if !ctx.self_voicing() {
            self.read = None;
            self.read_option = None;
            return;
        }
        if self.skipping {
            return;
        }
        let Some(event) = &self.current else {
            return;
        };

        let reading = Reading::of(event, |speaker| {
            ctx.characters.display_name(speaker, ctx.story)
        });
        if let Some(reading) = reading {
            let key = (ctx.log.len(), reading);
            if self.read.as_ref() != Some(&key) {
                ctx.read_aloud(&key.1);
                self.read = Some(key);
                self.read_option = match event {
                    Event::Choice { options } => options.iter().position(|option| option.enabled),
                    _ => None,
                };
            }
        }

        if let Event::Choice { options } = event
            && let Some(at) = self.choice_focus.index()
            && self.read_option != Some(at)
            && let Some(option) = options.get(at)
        {
            ctx.read_aloud(&Reading::Option {
                text: plain(&option.text),
            });
            self.read_option = Some(at);
        }
    }
}
