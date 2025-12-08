use crate::domain::value_objects::FizzyId;

/// Kanban column within a board
#[derive(Debug, Clone)]
pub struct Column {
    pub id: FizzyId,
    pub account_id: FizzyId,
    pub board_id: FizzyId,
    pub name: String,
    pub color: String,
    pub position: i32,
}

impl Column {
    /// Format column with color emoji for display
    pub fn formatted_name(&self) -> String {
        let color_emoji = match self.color.to_lowercase().as_str() {
            "red" => "🔴",
            "orange" => "🟠",
            "yellow" => "🟡",
            "green" => "🟢",
            "blue" => "🔵",
            "purple" => "🟣",
            "gray" | "grey" => "⚪",
            _ => "⬜",
        };
        format!("{} {}", color_emoji, self.name)
    }
}
