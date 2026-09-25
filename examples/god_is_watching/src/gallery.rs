use novn::data::session::{background_key, character_key, music_key};

pub const STUDY_DESK: &str = "cg:study_desk";

pub enum Art {
    Character(&'static str, &'static str),
    Place(&'static str),
    Moment(&'static str, &'static str, [f32; 4]),
    Track(&'static str),
}

pub struct Entry {
    pub art: Art,
    pub title: &'static str,
}

impl Entry {
    pub fn key(&self) -> String {
        match self.art {
            Art::Character(character, image) => character_key(character, image),
            Art::Place(background) => background_key(background),
            Art::Moment(key, _, _) => key.to_string(),
            Art::Track(track) => music_key(track),
        }
    }

    pub fn picture(&self) -> Option<String> {
        match self.art {
            Art::Character(character, image) => Some(format!("characters/{character}/{image}.png")),
            Art::Place(background) => Some(format!("backgrounds/{background}.png")),
            Art::Moment(_, picture, _) => Some(picture.to_string()),
            Art::Track(_) => None,
        }
    }

    pub fn crop(&self) -> Option<[f32; 4]> {
        match self.art {
            Art::Moment(_, _, crop) => Some(crop),
            _ => None,
        }
    }

    pub fn track(&self) -> Option<&'static str> {
        match self.art {
            Art::Track(track) => Some(track),
            _ => None,
        }
    }
}

pub struct Tab {
    pub name: &'static str,
    pub entries: &'static [Entry],
}

const PEOPLE: &[Entry] = &[
    Entry {
        art: Art::Character("mary", "tired"),
        title: "Mary, at the desk",
    },
    Entry {
        art: Art::Character("mary", "resolved"),
        title: "Mary, decided",
    },
    Entry {
        art: Art::Character("mary", "afraid"),
        title: "Mary, afraid",
    },
    Entry {
        art: Art::Character("hugo", "tired"),
        title: "Hugo, after the third letter",
    },
    Entry {
        art: Art::Character("adelaide", "neutral"),
        title: "Adelaide Roque",
    },
    Entry {
        art: Art::Character("clara", "unveiled"),
        title: "Sister Clara, unveiled",
    },
    Entry {
        art: Art::Character("moriarty", "older"),
        title: "Moriarty, older",
    },
    Entry {
        art: Art::Character("registrar", "stern"),
        title: "The Registrar",
    },
];

const PLACES: &[Entry] = &[
    Entry {
        art: Art::Place("von_lucis_study"),
        title: "The study",
    },
    Entry {
        art: Art::Moment(
            STUDY_DESK,
            "backgrounds/von_lucis_study.png",
            [0.02, 0.35, 0.42, 0.5],
        ),
        title: "The desk, examined",
    },
    Entry {
        art: Art::Place("von_lucis_hall"),
        title: "The hall",
    },
    Entry {
        art: Art::Place("von_lucis_bedroom"),
        title: "Mary's room",
    },
    Entry {
        art: Art::Place("archive_office"),
        title: "The Archive",
    },
    Entry {
        art: Art::Place("santa_ilde_courtyard"),
        title: "Santa Ilde, the courtyard",
    },
    Entry {
        art: Art::Place("santa_ilde_office"),
        title: "Santa Ilde, the office",
    },
    Entry {
        art: Art::Place("river_road"),
        title: "The river road",
    },
];

const MUSIC: &[Entry] = &[
    Entry {
        art: Art::Track("title"),
        title: "God Is Watching",
    },
    Entry {
        art: Art::Track("archive"),
        title: "Two floors below the street",
    },
    Entry {
        art: Art::Track("von_lucis"),
        title: "The Von Lucis house",
    },
    Entry {
        art: Art::Track("santa_ilde"),
        title: "Santa Ilde",
    },
    Entry {
        art: Art::Track("ending"),
        title: "What was nearly written",
    },
];

pub const TABS: &[Tab] = &[
    Tab {
        name: "PEOPLE",
        entries: PEOPLE,
    },
    Tab {
        name: "PLACES",
        entries: PLACES,
    },
    Tab {
        name: "MUSIC",
        entries: MUSIC,
    },
];

pub fn found(tab: &Tab, seen: impl Fn(&str) -> bool) -> usize {
    tab.entries
        .iter()
        .filter(|entry| seen(&entry.key()))
        .count()
}
