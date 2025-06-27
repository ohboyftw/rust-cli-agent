#[derive(Debug)]
pub struct AppState {
    pub goal: String,
    pub plan: Vec<String>,
    pub history: Vec<(String, String)>,
    pub current_step: usize,
}

impl AppState {
    pub fn new(goal: String) -> Self {
        Self { goal, plan: Vec::new(), history: Vec::new(), current_step: 0 }
    }

    pub fn add_history(&mut self, entry_type: &str, content: &str) {
        self.history.push((entry_type.to_string(), content.to_string()));
    }

    pub fn get_context(&self) -> String {
        let mut context = format!("The overall goal is: {}\n", self.goal);
        context.push_str("\n--- History & Context ---\n");
        if self.history.is_empty() {
            context.push_str("No actions have been taken yet.\n");
        } else {
            for (entry_type, content) in &self.history {
                let summarized = if content.len() > 500 { format!("{}...", &content[..500]) } else { content.clone() };
                context.push_str(&format!("[{}]\n{}\n---\n", entry_type, summarized));
            }
        }
        context
    }
}
