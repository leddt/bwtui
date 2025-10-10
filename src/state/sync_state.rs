/// State related to vault synchronization
#[derive(Debug)]
pub struct SyncState {
    pub syncing: bool,
    sync_animation_frame: u8,
}

impl SyncState {
    pub fn new() -> Self {
        Self {
            syncing: false,
            sync_animation_frame: 0,
        }
    }

    pub fn start(&mut self) {
        self.syncing = true;
        self.sync_animation_frame = 0;
    }

    pub fn stop(&mut self) {
        self.syncing = false;
    }

    pub fn advance_animation(&mut self) {
        if self.syncing {
            self.sync_animation_frame = (self.sync_animation_frame + 1) % 8;
        }
    }

    pub fn spinner(&self) -> &str {
        if !self.syncing {
            return "";
        }
        match self.sync_animation_frame {
            0 => "⠋",
            1 => "⠙",
            2 => "⠹",
            3 => "⠸",
            4 => "⠼",
            5 => "⠴",
            6 => "⠦",
            7 => "⠧",
            _ => "⠋",
        }
    }
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}

