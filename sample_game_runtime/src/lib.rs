use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleGameState {
    Title,
    InGame,
    Pause,
    Result,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleGameRunMode {
    BoxSurvival2d,
    Arena3d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleGameButtonAction {
    StartGame,
    Start3dArena,
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
            Self::Start3dArena => "sample.start_3d_arena",
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
            "sample.start_3d_arena" => Self::Start3dArena,
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SampleGameEffect {
    PlayClickSound,
    StartNewRun,
    StartNew3dRun,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleGameEvent {
    ButtonAction(SampleGameButtonAction),
    EscapePressed,
    RunCleared,
    RunFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SampleGameLoop {
    state: SampleGameState,
    run_mode: SampleGameRunMode,
}

impl Default for SampleGameLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl SampleGameLoop {
    pub fn new() -> Self {
        Self::from_state_and_mode(SampleGameState::Title, SampleGameRunMode::BoxSurvival2d)
    }

    pub fn from_state(state: SampleGameState) -> Self {
        Self::from_state_and_mode(state, SampleGameRunMode::BoxSurvival2d)
    }

    pub fn from_state_and_mode(state: SampleGameState, run_mode: SampleGameRunMode) -> Self {
        Self { state, run_mode }
    }

    pub fn state(&self) -> SampleGameState {
        self.state
    }

    pub fn run_mode(&self) -> SampleGameRunMode {
        self.run_mode
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
            SampleGameButtonAction::StartGame => {
                self.run_mode = SampleGameRunMode::BoxSurvival2d;
                self.state = SampleGameState::InGame;
                vec![
                    SampleGameEffect::PlayClickSound,
                    SampleGameEffect::StartNewRun,
                ]
            }
            SampleGameButtonAction::Start3dArena => {
                self.run_mode = SampleGameRunMode::Arena3d;
                self.state = SampleGameState::InGame;
                vec![
                    SampleGameEffect::PlayClickSound,
                    SampleGameEffect::StartNew3dRun,
                ]
            }
            SampleGameButtonAction::RetryGame => {
                self.state = SampleGameState::InGame;
                match self.run_mode {
                    SampleGameRunMode::BoxSurvival2d => vec![
                        SampleGameEffect::PlayClickSound,
                        SampleGameEffect::StartNewRun,
                    ],
                    SampleGameRunMode::Arena3d => vec![
                        SampleGameEffect::PlayClickSound,
                        SampleGameEffect::StartNew3dRun,
                    ],
                }
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
