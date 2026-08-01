//! Random human-friendly workspace name generator (e.g. `"bold_turing"`).

/// Generate a random workspace name in `adjective_noun` format.
///
/// Backed by the `petname` crate's curated word lists and RNG.
/// Example outputs: `"bold_turing"`, `"calm_lovelace"`.
#[must_use]
pub fn generate_name() -> String {
    petname::petname(2, "_").unwrap_or_else(|| "workspace".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_is_two_words_joined_by_underscore() {
        let name = generate_name();
        let mut parts = name.splitn(2, '_');
        let adj = parts.next().unwrap_or_default();
        let noun = parts.next().unwrap_or_default();
        assert!(!adj.is_empty(), "missing adjective part");
        assert!(!noun.is_empty(), "missing noun part");
    }

    #[test]
    fn generates_distinct_names() {
        // Curated word lists are large; 20 draws matching is effectively impossible.
        let names: std::collections::HashSet<_> = (0..20).map(|_| generate_name()).collect();
        assert!(names.len() > 1, "20 calls all returned the same name");
    }
}
