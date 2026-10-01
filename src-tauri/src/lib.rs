pub mod commands;
pub mod models;
pub mod utils;

pub use commands::*;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    Emitter,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let toggle_theme_i = MenuItem::with_id(app, "toggle_theme", "Toggle Theme", true, None::<&str>)?;
            let view_menu = Submenu::with_items(
                app,
                "View",
                true,
                &[&toggle_theme_i],
            )?;

            #[cfg(target_os = "macos")]
            let app_menu = Submenu::with_items(
                app,
                "Git Tree",
                true,
                &[
                    &PredefinedMenuItem::about(app, None, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::quit(app, None)?,
                ],
            )?;

            #[cfg(target_os = "macos")]
            let edit_menu = Submenu::with_items(
                app,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(app, None)?,
                    &PredefinedMenuItem::redo(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::cut(app, None)?,
                    &PredefinedMenuItem::copy(app, None)?,
                    &PredefinedMenuItem::paste(app, None)?,
                    &PredefinedMenuItem::select_all(app, None)?,
                ],
            )?;

            #[cfg(target_os = "macos")]
            let menu = Menu::with_items(app, &[&app_menu, &edit_menu, &view_menu])?;
            
            #[cfg(not(target_os = "macos"))]
            let menu = Menu::with_items(app, &[&view_menu])?;

            app.set_menu(menu)?;

            app.on_menu_event(move |app, event| {
                if event.id().as_ref() == "toggle_theme" {
                    let _ = app.emit("toggle-theme", ());
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_branches, get_remote_branches, get_tags, get_authors, get_git_profile, set_git_profile, get_contributor_report, get_stashes, get_commits, get_commit_details, get_commit_branches,
            get_file_diff, compare_file_commits, get_file_image_versions, get_uncommitted_changes, stage_file, unstage_file, commit_changes, fetch_remote, checkout_branch, delete_branch, delete_remote_branch, push_branch, pull_branch,
            merge_branch, rename_branch, create_branch, set_upstream, rebase_branch, fast_forward_merge,
            get_remotes, add_remote, remove_remote, cherry_pick, get_cherry_pick_status, continue_cherry_pick, skip_cherry_pick, abort_cherry_pick, reset_branch, drop_stash, stash_changes, pop_stash, get_tracked_files, clone_repository,
            get_remote_info, checkout_pull_request, get_file_blame,
            create_tag, push_tag, push_all_tags, delete_tag,
            get_worktrees, add_worktree, remove_worktree, lock_worktree, unlock_worktree, prune_worktrees,
            get_submodules, init_submodules, update_submodules, sync_submodules, add_submodule,
            run_git_terminal_command,
            get_rebase_commits, start_interactive_rebase, get_rebase_status, continue_rebase, skip_rebase, abort_rebase,
            apply_patch,
            get_conflicted_files, resolve_conflict_file, get_repository_operation_state, abort_repository_operation, continue_repository_operation,
            get_reflog, reset_to_reflog_entry, create_branch_from_reflog,
            purge_file_from_history,
            drop_commits_from_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
