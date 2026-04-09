use crate::app::HarnessKind;

pub fn recognize_harness(
    current_command: Option<&str>,
    title: &str,
    preview: &str,
) -> Option<HarnessKind> {
    let current_command = current_command.unwrap_or_default();

    if matches_any([current_command, title, preview], ["claude", "claude code"]) {
        return Some(HarnessKind::ClaudeCode);
    }

    if matches_any([current_command, title, preview], ["codex", "codex cli"]) {
        return Some(HarnessKind::CodexCli);
    }

    if matches_any([current_command, title, preview], ["gemini", "gemini cli"]) {
        return Some(HarnessKind::GeminiCli);
    }

    if matches_any([current_command, title, preview], ["opencode"]) {
        return Some(HarnessKind::OpenCode);
    }

    None
}

fn matches_any<'a>(
    haystacks: impl IntoIterator<Item = &'a str>,
    needles: impl IntoIterator<Item = &'a str>,
) -> bool {
    let haystacks = haystacks
        .into_iter()
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    needles.into_iter().any(|needle| {
        let needle = needle.to_ascii_lowercase();
        haystacks.iter().any(|haystack| haystack.contains(&needle))
    })
}

#[cfg(test)]
mod tests {
    use super::recognize_harness;
    use crate::app::HarnessKind;

    #[test]
    fn recognizes_supported_harnesses_from_command_title_and_preview() {
        assert_eq!(
            recognize_harness(Some("claude"), "shell", ""),
            Some(HarnessKind::ClaudeCode)
        );
        assert_eq!(
            recognize_harness(None, "codex-main", ""),
            Some(HarnessKind::CodexCli)
        );
        assert_eq!(
            recognize_harness(None, "shell", "Gemini CLI ready"),
            Some(HarnessKind::GeminiCli)
        );
        assert_eq!(
            recognize_harness(None, "shell", "OpenCode interactive shell"),
            Some(HarnessKind::OpenCode)
        );
    }

    #[test]
    fn returns_none_for_unrecognized_panes() {
        assert_eq!(recognize_harness(Some("zsh"), "notes", "plain shell"), None);
    }
}
