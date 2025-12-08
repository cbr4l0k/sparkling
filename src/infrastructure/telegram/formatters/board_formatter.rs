use crate::domain::entities::Board;

pub struct BoardFormatter;

impl BoardFormatter {
    /// Format a list of boards
    pub fn format_board_list(boards: &[Board]) -> String {
        if boards.is_empty() {
            return "No boards found.".to_string();
        }

        let lines: Vec<String> = boards
            .iter()
            .map(|board| {
                let access_indicator = if board.all_access { "🌐" } else { "🔒" };
                let card_count = board.card_count
                    .map(|c| format!(" ({} cards)", c))
                    .unwrap_or_default();

                format!("{} <b>{}</b>{}", access_indicator, escape_html(&board.name), card_count)
            })
            .collect();

        lines.join("\n")
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
