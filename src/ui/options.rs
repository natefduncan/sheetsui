//! The session's options and the `:set` command. Options live for the session
//! and are not saved. See `planning/search-design.md`, "Options".
//!
//! To add an option, add a row to [`OPTIONS`] and, if the code reads it, a
//! getter on [`Options`].

/// What kind of value an option holds, and its default.
#[derive(Debug, Clone, Copy)]
pub enum Kind {
    /// On or off.
    Bool(bool),
    /// One of a fixed list of words. The default is the first. No option
    /// uses it yet; `searchin` will.
    #[cfg_attr(not(test), allow(dead_code))]
    Choice(&'static [&'static str]),
}

/// One row of the options table.
#[derive(Debug, Clone, Copy)]
pub struct Spec {
    pub name: &'static str,
    /// Vim's short name, if it has one.
    pub short: Option<&'static str>,
    pub kind: Kind,
}

/// Every option there is.
pub const OPTIONS: &[Spec] = &[
    Spec {
        name: "wrapscan",
        short: Some("ws"),
        kind: Kind::Bool(true),
    },
    Spec {
        name: "ignorecase",
        short: Some("ic"),
        kind: Kind::Bool(false),
    },
    Spec {
        name: "smartcase",
        short: Some("scs"),
        kind: Kind::Bool(false),
    },
];

/// The current value of an option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Bool(bool),
    Choice(&'static str),
}

impl Kind {
    fn default_value(self) -> Value {
        match self {
            Kind::Bool(on) => Value::Bool(on),
            Kind::Choice(words) => Value::Choice(words[0]),
        }
    }
}

/// The value of every option in a table.
#[derive(Debug)]
pub struct Options {
    table: &'static [Spec],
    values: Vec<Value>,
}

impl Default for Options {
    fn default() -> Self {
        Self::new(OPTIONS)
    }
}

impl Options {
    /// Every option in `table` at its default.
    pub fn new(table: &'static [Spec]) -> Self {
        let values = table.iter().map(|s| s.kind.default_value()).collect();
        Self { table, values }
    }

    /// Whether a search wraps around the end of the sheet.
    pub fn wrapscan(&self) -> bool {
        self.flag("wrapscan")
    }

    /// Whether a pattern ignores case, unless `\c`, `\C` or `smartcase` say
    /// otherwise.
    pub fn ignorecase(&self) -> bool {
        self.flag("ignorecase")
    }

    /// Whether, with `ignorecase`, a pattern with an uppercase letter matches
    /// case.
    pub fn smartcase(&self) -> bool {
        self.flag("smartcase")
    }

    /// The value of a boolean option. Panics on a name not in the table,
    /// which is a bug in the getter, not a user error.
    fn flag(&self, name: &str) -> bool {
        match self.get(name) {
            Some(Value::Bool(on)) => on,
            other => panic!("{name} is not a boolean option: {other:?}"),
        }
    }

    /// The value of an option, by its long or short name.
    pub fn get(&self, name: &str) -> Option<Value> {
        self.index(name).map(|i| self.values[i])
    }

    fn index(&self, name: &str) -> Option<usize> {
        self.table
            .iter()
            .position(|s| s.name == name || s.short == Some(name))
    }

    /// Runs the argument of `:set`: one or more settings separated by spaces.
    /// Returns the text to show, if any: the values asked for, or every
    /// option for a bare `:set`. Settings before an error stay applied, as
    /// in vim.
    pub fn set(&mut self, args: &str) -> Result<Option<String>, String> {
        let mut shown = Vec::new();
        let mut any = false;
        for arg in args.split_whitespace() {
            any = true;
            if let Some(text) = self.set_one(arg)? {
                shown.push(text);
            }
        }
        if !any {
            shown = (0..self.table.len()).map(|i| self.show(i)).collect();
        }
        Ok(if shown.is_empty() {
            None
        } else {
            Some(shown.join("  "))
        })
    }

    fn set_one(&mut self, arg: &str) -> Result<Option<String>, String> {
        let invalid = || format!("E474: Invalid argument: {arg}");
        if let Some((name, value)) = arg.split_once('=') {
            let i = self.lookup(name)?;
            let Kind::Choice(words) = self.table[i].kind else {
                return Err(invalid());
            };
            let Some(word) = words.iter().find(|w| **w == value) else {
                return Err(format!(
                    "E474: Invalid argument: {arg} (one of: {})",
                    words.join(", ")
                ));
            };
            self.values[i] = Value::Choice(word);
            return Ok(None);
        }
        if let Some(name) = arg.strip_suffix('?') {
            return Ok(Some(self.show(self.lookup(name)?)));
        }
        if let Some(name) = arg.strip_suffix('!') {
            let i = self.lookup(name)?;
            let Value::Bool(on) = self.values[i] else {
                return Err(invalid());
            };
            self.values[i] = Value::Bool(!on);
            return Ok(None);
        }
        // `noname` turns a boolean off, unless an option is called that.
        if let Some(name) = arg.strip_prefix("no").filter(|_| self.index(arg).is_none()) {
            let i = self.lookup(name)?;
            if !matches!(self.values[i], Value::Bool(_)) {
                return Err(invalid());
            }
            self.values[i] = Value::Bool(false);
            return Ok(None);
        }
        let i = self.lookup(arg)?;
        match self.values[i] {
            Value::Bool(_) => {
                self.values[i] = Value::Bool(true);
                Ok(None)
            }
            // As in vim, naming a value option shows it.
            Value::Choice(_) => Ok(Some(self.show(i))),
        }
    }

    fn lookup(&self, name: &str) -> Result<usize, String> {
        self.index(name)
            .ok_or_else(|| format!("E518: Unknown option: {name}"))
    }

    /// An option as vim shows it: `wrapscan`, `nowrapscan`, `searchin=raw`.
    fn show(&self, i: usize) -> String {
        let name = self.table[i].name;
        match self.values[i] {
            Value::Bool(true) => name.to_owned(),
            Value::Bool(false) => format!("no{name}"),
            Value::Choice(word) => format!("{name}={word}"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// A table with a value option, since the real one has none yet.
    const TABLE: &[Spec] = &[
        Spec {
            name: "wrapscan",
            short: Some("ws"),
            kind: Kind::Bool(true),
        },
        Spec {
            name: "ignorecase",
            short: Some("ic"),
            kind: Kind::Bool(false),
        },
        Spec {
            name: "searchin",
            short: None,
            kind: Kind::Choice(&["value", "raw"]),
        },
    ];

    fn options() -> Options {
        Options::new(TABLE)
    }

    #[test]
    fn options_start_at_their_defaults() {
        let o = Options::default();
        assert!(o.wrapscan());
        assert!(!o.ignorecase());
        assert!(!o.smartcase());
        assert_eq!(Some(Value::Bool(false)), o.get("scs"));
        let o = options();
        assert_eq!(Some(Value::Bool(false)), o.get("ic"));
        assert_eq!(Some(Value::Choice("value")), o.get("searchin"));
    }

    #[test]
    fn booleans_turn_on_off_and_toggle_by_either_name() {
        let mut o = options();
        assert_eq!(Ok(None), o.set("nows"));
        assert!(!o.wrapscan());
        assert_eq!(Ok(None), o.set("wrapscan"));
        assert!(o.wrapscan());
        assert_eq!(Ok(None), o.set("ws!"));
        assert!(!o.wrapscan());
        assert_eq!(Ok(None), o.set("wrapscan!"));
        assert!(o.wrapscan());
        assert_eq!(Ok(None), o.set("ic"));
        assert_eq!(Some(Value::Bool(true)), o.get("ignorecase"));
    }

    #[test]
    fn a_question_mark_shows_the_value() {
        let mut o = options();
        assert_eq!(Ok(Some("wrapscan".to_owned())), o.set("ws?"));
        o.set("nows").unwrap();
        assert_eq!(Ok(Some("nowrapscan".to_owned())), o.set("wrapscan?"));
        assert_eq!(Ok(Some("searchin=value".to_owned())), o.set("searchin?"));
        // Naming a value option shows it too, as in vim.
        assert_eq!(Ok(Some("searchin=value".to_owned())), o.set("searchin"));
    }

    #[test]
    fn value_options_take_one_of_their_words() {
        let mut o = options();
        assert_eq!(Ok(None), o.set("searchin=raw"));
        assert_eq!(Some(Value::Choice("raw")), o.get("searchin"));
        let err = o.set("searchin=formula").unwrap_err();
        assert_eq!(
            "E474: Invalid argument: searchin=formula (one of: value, raw)",
            err
        );
        assert_eq!(Some(Value::Choice("raw")), o.get("searchin"));
    }

    #[test]
    fn a_form_that_does_not_fit_the_kind_is_an_error() {
        let mut o = options();
        assert_eq!(Err("E474: Invalid argument: ws=1".to_owned()), o.set("ws=1"));
        assert_eq!(
            Err("E474: Invalid argument: nosearchin".to_owned()),
            o.set("nosearchin")
        );
        assert_eq!(
            Err("E474: Invalid argument: searchin!".to_owned()),
            o.set("searchin!")
        );
        assert!(o.wrapscan());
    }

    #[test]
    fn an_unknown_option_is_e518() {
        let mut o = options();
        assert_eq!(Err("E518: Unknown option: bogus".to_owned()), o.set("bogus"));
        assert_eq!(Err("E518: Unknown option: bogus".to_owned()), o.set("nobogus"));
        assert_eq!(Err("E518: Unknown option: bogus".to_owned()), o.set("bogus?"));
        assert_eq!(Err("E518: Unknown option: bogus".to_owned()), o.set("bogus!"));
        assert_eq!(Err("E518: Unknown option: bogus".to_owned()), o.set("bogus=1"));
    }

    #[test]
    fn several_settings_apply_in_order_up_to_an_error() {
        let mut o = options();
        assert_eq!(
            Ok(Some("noignorecase  nowrapscan".to_owned())),
            o.set("nows ic! ic! ic? ws?")
        );
        assert!(o.set("ws bogus ic").is_err());
        assert!(o.wrapscan());
        assert_eq!(Some(Value::Bool(false)), o.get("ic"));
    }

    #[test]
    fn a_bare_set_shows_every_option() {
        let mut o = options();
        assert_eq!(
            Ok(Some("wrapscan  noignorecase  searchin=value".to_owned())),
            o.set("  ")
        );
    }
}
