use crate::domain::entities::Card;

pub struct CardFormatter;

impl CardFormatter {
    /// Format a single card for display
    pub fn format_card(card: &Card, base_url: Option<&str>) -> String {
        let mut lines = vec![format!(
            "{} <b>#{}</b> {}",
            card.status.emoji(),
            card.number,
            escape_html(&card.title)
        )];

        if let Some(board_name) = &card.board_name {
            lines.push(format!("Board: {}", escape_html(board_name)));
        }

        if let Some(column_name) = &card.column_name {
            lines.push(format!("Column: {}", escape_html(column_name)));
        }

        lines.push(format!("Status: {}", card.status.display_name()));

        if !card.assignee_names.is_empty() {
            lines.push(format!("Assignees: {}", card.assignee_names.join(", ")));
        }

        if !card.tag_titles.is_empty() {
            lines.push(format!("Tags: {}", card.tag_titles.join(", ")));
        }

        if let Some(due) = card.due_on {
            lines.push(format!("Due: {}", due));
        }

        if card.is_golden {
            lines.push("⭐ Golden".to_string());
        }

        if let Some(url) = card.web_url(base_url) {
            lines.push(format!("\n<a href=\"{}\">Open in Fizzy</a>", url));
        }

        lines.join("\n")
    }

    /// Format a list of cards
    pub fn format_card_list(cards: &[Card]) -> String {
        if cards.is_empty() {
            return "No cards found.".to_string();
        }

        let lines: Vec<String> = cards
            .iter()
            .map(|card| {
                let due_str = card
                    .due_on
                    .map(|d| format!(" 📅 {}", d))
                    .unwrap_or_default();

                format!(
                    "{} <b>#{}</b> {}{}",
                    card.status.emoji(),
                    card.number,
                    escape_html(&card.title),
                    due_str
                )
            })
            .collect();

        lines.join("\n")
    }
}

/// Escape HTML special characters for Telegram
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
