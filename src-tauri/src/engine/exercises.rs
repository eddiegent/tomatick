use serde::{Deserialize, Serialize};

/// One entry of the break exercise list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exercise {
    pub name: String,
    /// Optional duration or repetition count, free text such as "30 s".
    #[serde(default)]
    pub amount: Option<String>,
}

impl Exercise {
    /// Text shown to the user, for example `Planks (30 s)`.
    pub fn label(&self) -> String {
        match self.amount.as_deref().map(str::trim) {
            Some(amount) if !amount.is_empty() => format!("{} ({amount})", self.name.trim()),
            _ => self.name.trim().to_string(),
        }
    }
}

/// The ordered exercise list and the position of the rotation in it.
#[derive(Clone, Debug, Default)]
pub struct Rotation {
    items: Vec<Exercise>,
    next: usize,
}

impl Rotation {
    pub fn new(items: Vec<Exercise>) -> Self {
        let mut rotation = Self::default();
        rotation.set_items(items);
        rotation
    }

    /// Replaces the list. Entries with an empty name are dropped. The
    /// rotation position is kept when it still fits, otherwise it restarts.
    pub fn set_items(&mut self, items: Vec<Exercise>) {
        self.items = items
            .into_iter()
            .filter(|item| !item.name.trim().is_empty())
            .collect();
        if self.next >= self.items.len() {
            self.next = 0;
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Index of the exercise that comes next, for saving between launches.
    pub fn position(&self) -> usize {
        self.next
    }

    /// Puts the rotation back where it was. A position that no longer fits
    /// the list restarts at the first exercise.
    pub fn set_position(&mut self, position: usize) {
        self.next = if position < self.items.len() {
            position
        } else {
            0
        };
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the next exercise label and advances, wrapping at the end.
    /// Returns `None` when the list is empty.
    pub fn next_label(&mut self) -> Option<String> {
        let item = self.items.get(self.next)?;
        let label = item.label();
        self.next = (self.next + 1) % self.items.len();
        Some(label)
    }
}
