#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleGameState {
    Title,
    InGame,
    Pause,
    Result,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleGameButtonAction {
    StartGame,
    ResumeGame,
    RetryGame,
    BackToTitle,
    MasterVolumeDown,
    MasterVolumeUp,
    BgmVolumeDown,
    BgmVolumeUp,
    SeVolumeDown,
    SeVolumeUp,
    ToggleFullscreen,
    ExitGame,
}

impl SampleGameButtonAction {
    pub fn action_id(self) -> &'static str {
        match self {
            Self::StartGame => "sample.start_game",
            Self::ResumeGame => "sample.resume_game",
            Self::RetryGame => "sample.retry_game",
            Self::BackToTitle => "sample.back_to_title",
            Self::MasterVolumeDown => "sample.master_volume_down",
            Self::MasterVolumeUp => "sample.master_volume_up",
            Self::BgmVolumeDown => "sample.bgm_volume_down",
            Self::BgmVolumeUp => "sample.bgm_volume_up",
            Self::SeVolumeDown => "sample.se_volume_down",
            Self::SeVolumeUp => "sample.se_volume_up",
            Self::ToggleFullscreen => "sample.toggle_fullscreen",
            Self::ExitGame => "sample.exit_game",
        }
    }

    pub fn from_action_id(action_id: &str) -> Option<Self> {
        Some(match action_id {
            "sample.start_game" => Self::StartGame,
            "sample.resume_game" => Self::ResumeGame,
            "sample.retry_game" => Self::RetryGame,
            "sample.back_to_title" => Self::BackToTitle,
            "sample.master_volume_down" => Self::MasterVolumeDown,
            "sample.master_volume_up" => Self::MasterVolumeUp,
            "sample.bgm_volume_down" => Self::BgmVolumeDown,
            "sample.bgm_volume_up" => Self::BgmVolumeUp,
            "sample.se_volume_down" => Self::SeVolumeDown,
            "sample.se_volume_up" => Self::SeVolumeUp,
            "sample.toggle_fullscreen" => Self::ToggleFullscreen,
            "sample.exit_game" => Self::ExitGame,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SampleGameEffect {
    PlayClickSound,
    StartNewRun,
    ResumeRun,
    SetupPauseMenu,
    SetupResultMenu,
    SetupTitleScreen,
    AdjustMasterVolume(f32),
    AdjustBgmVolume(f32),
    AdjustSeVolume(f32),
    ToggleFullscreen,
    RequestWindowClose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleGameEvent {
    ButtonAction(SampleGameButtonAction),
    EscapePressed,
    RunCleared,
    RunFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleGameLoop {
    state: SampleGameState,
}

impl Default for SampleGameLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl SampleGameLoop {
    pub fn new() -> Self {
        Self {
            state: SampleGameState::Title,
        }
    }

    pub fn state(&self) -> SampleGameState {
        self.state
    }

    pub fn dispatch(&mut self, event: SampleGameEvent) -> Vec<SampleGameEffect> {
        match event {
            SampleGameEvent::ButtonAction(action) => self.handle_button_action(action),
            SampleGameEvent::EscapePressed => self.handle_escape_pressed(),
            SampleGameEvent::RunCleared | SampleGameEvent::RunFailed => {
                self.state = SampleGameState::Result;
                vec![
                    SampleGameEffect::PlayClickSound,
                    SampleGameEffect::SetupResultMenu,
                ]
            }
        }
    }

    fn handle_button_action(&mut self, action: SampleGameButtonAction) -> Vec<SampleGameEffect> {
        match action {
            SampleGameButtonAction::StartGame | SampleGameButtonAction::RetryGame => {
                self.state = SampleGameState::InGame;
                vec![
                    SampleGameEffect::PlayClickSound,
                    SampleGameEffect::StartNewRun,
                ]
            }
            SampleGameButtonAction::ResumeGame => {
                self.state = SampleGameState::InGame;
                vec![
                    SampleGameEffect::PlayClickSound,
                    SampleGameEffect::ResumeRun,
                ]
            }
            SampleGameButtonAction::BackToTitle => {
                self.state = SampleGameState::Title;
                vec![
                    SampleGameEffect::PlayClickSound,
                    SampleGameEffect::SetupTitleScreen,
                ]
            }
            SampleGameButtonAction::MasterVolumeDown => {
                vec![
                    SampleGameEffect::PlayClickSound,
                    SampleGameEffect::AdjustMasterVolume(-0.1),
                ]
            }
            SampleGameButtonAction::MasterVolumeUp => {
                vec![
                    SampleGameEffect::PlayClickSound,
                    SampleGameEffect::AdjustMasterVolume(0.1),
                ]
            }
            SampleGameButtonAction::BgmVolumeDown => {
                vec![
                    SampleGameEffect::PlayClickSound,
                    SampleGameEffect::AdjustBgmVolume(-0.1),
                ]
            }
            SampleGameButtonAction::BgmVolumeUp => {
                vec![
                    SampleGameEffect::PlayClickSound,
                    SampleGameEffect::AdjustBgmVolume(0.1),
                ]
            }
            SampleGameButtonAction::SeVolumeDown => {
                vec![
                    SampleGameEffect::PlayClickSound,
                    SampleGameEffect::AdjustSeVolume(-0.1),
                ]
            }
            SampleGameButtonAction::SeVolumeUp => {
                vec![
                    SampleGameEffect::PlayClickSound,
                    SampleGameEffect::AdjustSeVolume(0.1),
                ]
            }
            SampleGameButtonAction::ToggleFullscreen => {
                vec![
                    SampleGameEffect::PlayClickSound,
                    SampleGameEffect::ToggleFullscreen,
                ]
            }
            SampleGameButtonAction::ExitGame => {
                vec![
                    SampleGameEffect::PlayClickSound,
                    SampleGameEffect::RequestWindowClose,
                ]
            }
        }
    }

    fn handle_escape_pressed(&mut self) -> Vec<SampleGameEffect> {
        match self.state {
            SampleGameState::InGame => {
                self.state = SampleGameState::Pause;
                vec![SampleGameEffect::SetupPauseMenu]
            }
            SampleGameState::Pause => {
                self.state = SampleGameState::InGame;
                vec![SampleGameEffect::ResumeRun]
            }
            SampleGameState::Title | SampleGameState::Result => Vec::new(),
        }
    }
}
