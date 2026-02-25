/// Options extracted from a `worktree://` deep link.
#[derive(Debug, Clone, Default)]
pub struct DeepLinkOptions {
    /// Editor override from the `editor` query param. May be a symbolic name
    /// (`cursor`, `code`, `zed`, `nvim`, etc.) or a raw percent-decoded command.
    pub editor: Option<String>,
}
