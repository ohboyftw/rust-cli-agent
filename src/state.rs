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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let goal = "Test goal".to_string();
        let state = AppState::new(goal.clone());

        assert_eq!(state.goal, goal);
        assert_eq!(state.plan, Vec::<String>::new());
        assert_eq!(state.history, Vec::<(String, String)>::new());
        assert_eq!(state.current_step, 0);
    }

    #[test]
    fn test_add_history() {
        let mut state = AppState::new("Test goal".to_string());
        
        state.add_history("Tool", "Tool output");
        state.add_history("Code", "Generated code");

        assert_eq!(state.history.len(), 2);
        assert_eq!(state.history[0], ("Tool".to_string(), "Tool output".to_string()));
        assert_eq!(state.history[1], ("Code".to_string(), "Generated code".to_string()));
    }

    #[test]
    fn test_get_context_empty_history() {
        let state = AppState::new("Test goal".to_string());
        let context = state.get_context();

        assert!(context.contains("The overall goal is: Test goal"));
        assert!(context.contains("--- History & Context ---"));
        assert!(context.contains("No actions have been taken yet."));
    }

    #[test]
    fn test_get_context_with_history() {
        let mut state = AppState::new("Test goal".to_string());
        state.add_history("Tool", "Tool output");
        state.add_history("Code", "Generated code");

        let context = state.get_context();

        assert!(context.contains("The overall goal is: Test goal"));
        assert!(context.contains("--- History & Context ---"));
        assert!(context.contains("[Tool]"));
        assert!(context.contains("Tool output"));
        assert!(context.contains("[Code]"));
        assert!(context.contains("Generated code"));
        assert!(!context.contains("No actions have been taken yet."));
    }

    #[test]
    fn test_get_context_with_long_content() {
        let mut state = AppState::new("Test goal".to_string());
        let long_content = "a".repeat(600); // Content longer than 500 chars
        state.add_history("LongContent", &long_content);

        let context = state.get_context();

        assert!(context.contains("[LongContent]"));
        assert!(context.contains(&"a".repeat(500))); // Should be truncated to 500 chars
        assert!(context.contains("...")); // Should have ellipsis
        assert!(!context.contains(&long_content)); // Should not contain full content
    }

    #[test]
    fn test_state_debug() {
        let state = AppState::new("Test goal".to_string());
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("AppState"));
        assert!(debug_str.contains("goal"));
        assert!(debug_str.contains("plan"));
        assert!(debug_str.contains("history"));
        assert!(debug_str.contains("current_step"));
    }

    #[test]
    fn test_plan_modification() {
        let mut state = AppState::new("Test goal".to_string());
        state.plan.push("Step 1".to_string());
        state.plan.push("Step 2".to_string());
        state.current_step = 1;

        assert_eq!(state.plan.len(), 2);
        assert_eq!(state.plan[0], "Step 1");
        assert_eq!(state.plan[1], "Step 2");
        assert_eq!(state.current_step, 1);
    }

    #[test]
    fn test_multiple_history_entries() {
        let mut state = AppState::new("Complex goal".to_string());
        
        for i in 0..5 {
            state.add_history(&format!("Type{}", i), &format!("Content{}", i));
        }

        assert_eq!(state.history.len(), 5);
        let context = state.get_context();
        
        for i in 0..5 {
            assert!(context.contains(&format!("[Type{}]", i)));
            assert!(context.contains(&format!("Content{}", i)));
        }
    }
}
