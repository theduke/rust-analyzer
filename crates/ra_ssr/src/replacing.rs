//! Code for applying replacement templates for matches that have previously been found.

use crate::matching::Var;
use crate::parsing::PatternElement;
use crate::{Match, SsrMatches};
use ra_syntax::ast::AstToken;
use ra_syntax::TextSize;
use ra_text_edit::TextEdit;

/// Returns a text edit that will replace each match in `matches` with its corresponding replacement
/// template. Placeholders in the template will have been substituted with whatever they matched to
/// in the original code.
pub(crate) fn matches_to_edit(matches: &SsrMatches) -> TextEdit {
    matches_to_edit_at_offset(matches, 0.into())
}

fn matches_to_edit_at_offset(matches: &SsrMatches, relative_start: TextSize) -> TextEdit {
    let mut edit_builder = ra_text_edit::TextEditBuilder::default();
    for m in &matches.matches {
        edit_builder.replace(m.range.checked_sub(relative_start).unwrap(), render_replace(m));
    }
    edit_builder.finish()
}

fn render_replace(match_info: &Match) -> String {
    let mut out = String::new();
    let match_start = match_info.matched_node.text_range().start();
    for r in &match_info.template.tokens {
        match r {
            PatternElement::Token(t) => out.push_str(t.text.as_str()),
            PatternElement::Placeholder(p) => {
                if let Some(placeholder_value) =
                    match_info.placeholder_values.get(&Var(p.ident.to_string()))
                {
                    let range = &placeholder_value.range.range;
                    let mut matched_text = if let Some(node) = &placeholder_value.node {
                        node.text().to_string()
                    } else {
                        let relative_range = range.checked_sub(match_start).unwrap();
                        match_info.matched_node.text().to_string()
                            [usize::from(relative_range.start())..usize::from(relative_range.end())]
                            .to_string()
                    };
                    let edit =
                        matches_to_edit_at_offset(&placeholder_value.inner_matches, range.start());
                    edit.apply(&mut matched_text);
                    out.push_str(&matched_text);
                } else {
                    // We validated that all placeholder references were valid before we
                    // started, so this shouldn't happen.
                    panic!(
                        "Internal error: replacement referenced unknown placeholder {}",
                        p.ident
                    );
                }
            }
        }
    }
    for comment in &match_info.ignored_comments {
        out.push_str(&comment.syntax().to_string());
    }
    out
}
