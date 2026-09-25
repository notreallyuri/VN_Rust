use std::cell::Cell;

thread_local! {
    static REDUCED: Cell<bool> = const { Cell::new(false) };
}

pub fn reduced() -> bool {
    REDUCED.with(Cell::get)
}

pub fn set_reduced(on: bool) {
    REDUCED.with(|reduced| reduced.set(on));
}

pub fn calm(kind: novn_script::TransitionKind) -> novn_script::TransitionKind {
    use novn_script::TransitionKind;
    match kind {
        TransitionKind::SlideLeft | TransitionKind::SlideRight if reduced() => {
            TransitionKind::Dissolve
        }
        other => other,
    }
}
