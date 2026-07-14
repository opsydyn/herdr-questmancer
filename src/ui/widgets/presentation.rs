use std::borrow::Cow;

use crate::app::CharacterSet;

pub(super) fn present(value: &str, character_set: CharacterSet) -> Cow<'_, str> {
    let needs_replacement = value.chars().any(|character| {
        character.is_control() || (character_set == CharacterSet::Ascii && !character.is_ascii())
    });
    if !needs_replacement {
        return Cow::Borrowed(value);
    }

    Cow::Owned(
        value
            .chars()
            .map(|character| {
                if character.is_control() {
                    if character.is_whitespace() { ' ' } else { '?' }
                } else if character_set == CharacterSet::Ascii && !character.is_ascii() {
                    '?'
                } else {
                    character
                }
            })
            .collect(),
    )
}
