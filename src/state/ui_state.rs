use ratatui::layout::Rect;

/// State related to UI modes, dialogs, and layout
#[derive(Debug)]
pub struct UIState {
    pub details_panel_visible: bool,
    pub password_input_mode: bool,
    pub password_input: String,
    pub unlock_error: Option<String>,
    pub offer_save_token: bool,
    pub save_token_response: Option<bool>,
    pub show_not_logged_in_error: bool,
    pub list_area: Rect,
    pub details_panel_area: Rect,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            details_panel_visible: false,
            password_input_mode: false,
            password_input: String::new(),
            unlock_error: None,
            offer_save_token: false,
            save_token_response: None,
            show_not_logged_in_error: false,
            list_area: Rect::default(),
            details_panel_area: Rect::default(),
        }
    }

    pub fn toggle_details_panel(&mut self) {
        self.details_panel_visible = !self.details_panel_visible;
    }

    pub fn enter_password_mode(&mut self) {
        self.password_input_mode = true;
        self.password_input.clear();
        self.unlock_error = None;
    }

    pub fn exit_password_mode(&mut self) {
        self.password_input_mode = false;
        self.password_input.clear();
        self.unlock_error = None;
    }

    pub fn append_password_char(&mut self, c: char) {
        self.password_input.push(c);
    }

    pub fn delete_password_char(&mut self) {
        self.password_input.pop();
    }

    pub fn get_password(&self) -> String {
        self.password_input.clone()
    }

    pub fn set_unlock_error(&mut self, error: String) {
        self.unlock_error = Some(error);
    }

    pub fn enter_save_token_prompt(&mut self) {
        self.offer_save_token = true;
        self.save_token_response = None;
    }

    pub fn set_save_token_response(&mut self, response: bool) {
        self.save_token_response = Some(response);
    }

    pub fn exit_save_token_prompt(&mut self) {
        self.offer_save_token = false;
        self.save_token_response = None;
    }

    pub fn show_not_logged_in_popup(&mut self) {
        self.show_not_logged_in_error = true;
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}

