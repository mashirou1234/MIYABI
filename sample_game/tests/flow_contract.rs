use sample_game::{
    SampleGameButtonAction, SampleGameEffect, SampleGameEvent, SampleGameLoop, SampleGameRunMode,
    SampleGameState,
};

#[test]
fn sample_game_button_actions_round_trip_to_action_ids() {
    let actions = [
        SampleGameButtonAction::StartGame,
        SampleGameButtonAction::Start3dArena,
        SampleGameButtonAction::ResumeGame,
        SampleGameButtonAction::RetryGame,
        SampleGameButtonAction::BackToTitle,
        SampleGameButtonAction::MasterVolumeDown,
        SampleGameButtonAction::MasterVolumeUp,
        SampleGameButtonAction::BgmVolumeDown,
        SampleGameButtonAction::BgmVolumeUp,
        SampleGameButtonAction::SeVolumeDown,
        SampleGameButtonAction::SeVolumeUp,
        SampleGameButtonAction::ToggleFullscreen,
        SampleGameButtonAction::ExitGame,
    ];

    for action in actions {
        assert_eq!(
            SampleGameButtonAction::from_action_id(action.action_id()),
            Some(action),
            "action id must round-trip for {action:?}"
        );
    }

    assert_eq!(
        SampleGameButtonAction::from_action_id("sample.unknown"),
        None
    );
}

#[test]
fn sample_game_loop_covers_title_pause_result_and_exit_flow() {
    let mut game_loop = SampleGameLoop::new();
    assert_eq!(game_loop.state(), SampleGameState::Title);
    assert_eq!(game_loop.run_mode(), SampleGameRunMode::BoxSurvival2d);

    assert_eq!(
        game_loop.dispatch(SampleGameEvent::ButtonAction(
            SampleGameButtonAction::Start3dArena
        )),
        vec![
            SampleGameEffect::PlayClickSound,
            SampleGameEffect::StartNew3dRun,
        ]
    );
    assert_eq!(game_loop.state(), SampleGameState::InGame);
    assert_eq!(game_loop.run_mode(), SampleGameRunMode::Arena3d);

    assert_eq!(
        game_loop.dispatch(SampleGameEvent::ButtonAction(
            SampleGameButtonAction::BackToTitle
        )),
        vec![
            SampleGameEffect::PlayClickSound,
            SampleGameEffect::SetupTitleScreen,
        ]
    );
    assert_eq!(game_loop.state(), SampleGameState::Title);

    assert_eq!(
        game_loop.dispatch(SampleGameEvent::ButtonAction(
            SampleGameButtonAction::StartGame
        )),
        vec![
            SampleGameEffect::PlayClickSound,
            SampleGameEffect::StartNewRun,
        ]
    );
    assert_eq!(game_loop.state(), SampleGameState::InGame);
    assert_eq!(game_loop.run_mode(), SampleGameRunMode::BoxSurvival2d);

    assert_eq!(
        game_loop.dispatch(SampleGameEvent::EscapePressed),
        vec![SampleGameEffect::SetupPauseMenu]
    );
    assert_eq!(game_loop.state(), SampleGameState::Pause);

    assert_eq!(
        game_loop.dispatch(SampleGameEvent::ButtonAction(
            SampleGameButtonAction::ResumeGame
        )),
        vec![
            SampleGameEffect::PlayClickSound,
            SampleGameEffect::ResumeRun,
        ]
    );
    assert_eq!(game_loop.state(), SampleGameState::InGame);

    assert_eq!(
        game_loop.dispatch(SampleGameEvent::RunFailed),
        vec![
            SampleGameEffect::PlayClickSound,
            SampleGameEffect::SetupResultMenu,
        ]
    );
    assert_eq!(game_loop.state(), SampleGameState::Result);

    assert_eq!(
        game_loop.dispatch(SampleGameEvent::ButtonAction(
            SampleGameButtonAction::BackToTitle
        )),
        vec![
            SampleGameEffect::PlayClickSound,
            SampleGameEffect::SetupTitleScreen,
        ]
    );
    assert_eq!(game_loop.state(), SampleGameState::Title);

    assert_eq!(
        game_loop.dispatch(SampleGameEvent::ButtonAction(
            SampleGameButtonAction::ExitGame
        )),
        vec![
            SampleGameEffect::PlayClickSound,
            SampleGameEffect::RequestWindowClose,
        ]
    );
    assert_eq!(game_loop.state(), SampleGameState::Title);
}

#[test]
fn sample_game_loop_retries_last_3d_run_after_result() {
    let mut game_loop = SampleGameLoop::new();

    assert_eq!(
        game_loop.dispatch(SampleGameEvent::ButtonAction(
            SampleGameButtonAction::Start3dArena
        )),
        vec![
            SampleGameEffect::PlayClickSound,
            SampleGameEffect::StartNew3dRun,
        ]
    );
    assert_eq!(game_loop.run_mode(), SampleGameRunMode::Arena3d);

    assert_eq!(
        game_loop.dispatch(SampleGameEvent::RunCleared),
        vec![
            SampleGameEffect::PlayClickSound,
            SampleGameEffect::SetupResultMenu,
        ]
    );
    assert_eq!(game_loop.state(), SampleGameState::Result);

    assert_eq!(
        game_loop.dispatch(SampleGameEvent::ButtonAction(
            SampleGameButtonAction::RetryGame
        )),
        vec![
            SampleGameEffect::PlayClickSound,
            SampleGameEffect::StartNew3dRun,
        ]
    );
    assert_eq!(game_loop.state(), SampleGameState::InGame);
    assert_eq!(game_loop.run_mode(), SampleGameRunMode::Arena3d);
}
