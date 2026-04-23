use foreman::app::{
    inventory, reduce, Action, AgentStatus, AppState, HarnessKind, PaneBuilder, SelectionTarget,
    SessionBuilder, WindowBuilder,
};
use foreman::services::pull_requests::{
    PullRequestLookup, PullRequestService, SystemPullRequestBackend,
};
use tempfile::tempdir;

#[test]
fn non_repository_workspace_fails_soft_without_breaking_dashboard_state() {
    let workspace = tempdir().expect("temp dir should exist");
    let inventory = inventory([SessionBuilder::new("alpha").window(
        WindowBuilder::new("alpha:agents").pane(
            PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                .working_dir(workspace.path())
                .status(AgentStatus::Working),
        ),
    )]);
    let mut state = AppState::with_inventory(inventory);
    state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));

    let service = PullRequestService::new(SystemPullRequestBackend::new());
    let lookup = service
        .lookup(workspace.path())
        .expect("non-repository workspace should fail soft");

    assert_eq!(lookup, PullRequestLookup::Missing);

    reduce(
        &mut state,
        Action::SetPullRequestLookup {
            workspace_path: workspace.path().to_path_buf(),
            lookup,
        },
    );

    assert!(state.selected_pull_request().is_none());
    assert!(!state.is_pull_request_detail_open());
    assert_eq!(
        state.selection,
        Some(SelectionTarget::Pane("alpha:claude".into()))
    );
}
