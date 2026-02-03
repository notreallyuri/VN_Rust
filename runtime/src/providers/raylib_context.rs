use std::{cell::RefCell, rc::Rc};

use raylib::prelude::*;

#[derive(Clone)]
pub struct RaylibCtx {
    pub rl: Rc<RefCell<RaylibHandle>>,
    pub thread: Rc<RaylibThread>,
}

impl RaylibCtx {
    pub fn new(rl: RaylibHandle, thread: RaylibThread) -> Self {
        Self {
            rl: Rc::new(RefCell::new(rl)),
            thread: Rc::new(thread),
        }
    }
}
