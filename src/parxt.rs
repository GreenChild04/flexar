use std::{rc::Rc, fmt::Display};

use crate::{flext::Flext, token_node::Token, cursor::Position};

/// Parser context for parsing
#[derive(Debug, Clone, Copy)]
pub struct Parxt<'a, TT: Display> {
    pub tokens: &'a [Token<TT>],
    pub idx: u16,
    pub done: bool,
}

impl<'a, TT: Display> Parxt<'a, TT> {
    #[inline]
    pub fn new(tokens: &'a [Token<TT>]) -> Self {
        Self {
            tokens,
            idx: 0,
            done: tokens.is_empty(),
        }
    }

    #[inline]
    pub fn current_token(&self) -> Option<&'a Token<TT>> {
        self.tokens.get(self.idx as usize)
    }

    #[inline]
    pub fn current(&self) -> Option<&'a TT> {
        self.current_token().map(|x| &x.token_type)
    }

    #[inline]
    fn get_last_pos(&self) -> Position {
        if self.tokens.is_empty() { panic!("file's empty, gonna add code to handle that later") }
        let mut position = self.tokens[self.tokens.len() -1].position.clone();
        let mut end = (*position.1).clone();
        end.ln_idx += 1;
        let end = Rc::new(end);
        (position.0, position.1) = (end.clone(), end);
        position
    }
}

impl<'a, TT: Display> Flext for Parxt<'a, TT> {
    /// Advances to the next token
    #[inline]
    fn advance(&mut self) {
        self.idx += 1;
    }

    /// Un-Advances
    #[inline]
    fn revance(&mut self) {
        if self.idx != 0 {
            self.idx -= 1;
        }
    }

    /// Spawns a child flext
    #[inline]
    fn spawn(&self) -> Self {
        Self { tokens: self.tokens, idx: self.idx, done: self.done, }
    }

    /// Gets the current position of the cursor
    #[inline]
    fn position(&self) -> Position {
        self.current_token().map(|x| x.position.clone())
            .unwrap_or_else(|| self.get_last_pos())
    }
}