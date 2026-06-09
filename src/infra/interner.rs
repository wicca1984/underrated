//! String interner for deduping strings like tag names and attribute names.
// spec: S-0. infra (string interning)

#![allow(dead_code)]

use std::collections::HashMap;

/// A handle to an interned string.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Symbol(u32);

/// A string interner that maps strings to unique Symbols and vice versa.
pub struct Interner {
    map: HashMap<String, Symbol>,
    strings: Vec<String>,
}

impl Interner {
    /// Creates a new, empty Interner.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            strings: Vec::new(),
        }
    }

    /// Interns a string and returns its unique Symbol.
    /// If the string was already interned, the existing Symbol is returned.
    pub fn intern(&mut self, s: &str) -> Symbol {
        if let Some(&sym) = self.map.get(s) {
            return sym;
        }

        let sym = Symbol(self.strings.len() as u32);
        let s_owned = s.to_string();
        self.map.insert(s_owned.clone(), sym);
        self.strings.push(s_owned);
        sym
    }

    /// Resolves a Symbol back to its original string.
    /// Returns `None` if the Symbol does not belong to this interner.
    pub fn resolve(&self, sym: Symbol) -> Option<&str> {
        self.strings.get(sym.0 as usize).map(|s| s.as_str())
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_same_string() {
        let mut interner = Interner::new();
        let s1 = interner.intern("html");
        let s2 = interner.intern("html");
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_intern_different_strings() {
        let mut interner = Interner::new();
        let s1 = interner.intern("html");
        let s2 = interner.intern("body");
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_resolve() {
        let mut interner = Interner::new();
        let s1 = interner.intern("html");
        let s2 = interner.intern("body");
        assert_eq!(interner.resolve(s1), Some("html"));
        assert_eq!(interner.resolve(s2), Some("body"));
    }

    #[test]
    fn test_symbol_is_copy() {
        let mut interner = Interner::new();
        let s1 = interner.intern("html");
        let s2 = s1; // Copy
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_intern_twice_does_not_grow_storage() {
        let mut interner = Interner::new();
        interner.intern("html");
        let len1 = interner.strings.len();
        interner.intern("html");
        let len2 = interner.strings.len();
        assert_eq!(len1, len2);
    }
}
