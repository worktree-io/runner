use worktree_io::workspace::Workspace;

pub fn report_workspace(workspace: &Workspace, json: bool) {
    if json {
        println!(
            "{{\"path\":{:?},\"created\":{}}}",
            workspace.path.display().to_string(),
            workspace.created
        );
    } else if workspace.created {
        eprintln!("Created workspace at {}", workspace.path.display());
    } else {
        eprintln!("Workspace already exists at {}", workspace.path.display());
    }
}
