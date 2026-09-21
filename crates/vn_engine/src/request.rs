use crate::overlay::OverlayRequest;
use crate::ui::cursor::CursorKind;
use crate::ui::toast::Toast;

#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    Overlay(OverlayRequest),
    Toast(Toast),
    Tooltip(String),
    Cursor(CursorKind),
    Autosave,
    Screenshot,
}

#[derive(Debug, Default)]
pub struct Requests(Vec<Request>);

impl Requests {
    pub fn push(&mut self, request: Request) {
        self.0.push(request);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Request> {
        self.0.iter()
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Request> + use<'_> {
        self.0.drain(..)
    }

    pub fn resolve(&mut self) -> Resolved {
        let mut resolved = Resolved::default();
        for request in self.0.drain(..) {
            match request {
                Request::Overlay(overlay) => resolved.overlays.push(overlay),
                Request::Tooltip(text) => resolved.tooltip = Some(text),
                Request::Cursor(kind) => resolved.cursor = resolved.cursor.max(kind),
                Request::Toast(toast) => resolved.toast = Some(toast),
                Request::Autosave => resolved.autosave = true,
                Request::Screenshot => resolved.screenshot = true,
            }
        }
        resolved
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Resolved {
    pub overlays: Vec<OverlayRequest>,
    pub tooltip: Option<String>,
    pub toast: Option<Toast>,
    pub cursor: CursorKind,
    pub autosave: bool,
    pub screenshot: bool,
}
