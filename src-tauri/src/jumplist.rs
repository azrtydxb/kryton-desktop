#![cfg(target_os = "windows")]

use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::UI::Shell::{
    DestinationList, EnumerableObjectCollection, ICustomDestinationList, IShellLinkW,
    Common::IObjectCollection,
};
use windows_core::{w, Interface, PCWSTR};

/// A recent note entry for the jump list.
pub struct RecentNote {
    pub title: String,
    pub account_id: String,
    pub note_path: String,
}

/// Build a wide-string (null-terminated UTF-16) from a Rust &str.
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Create an `IShellLinkW` that opens `kryton://note/<path>?account=<id>`.
unsafe fn make_note_link(note: &RecentNote) -> windows_core::Result<IShellLinkW> {
    let link: IShellLinkW =
        CoCreateInstance(&windows::Win32::UI::Shell::ShellLink, None, CLSCTX_INPROC_SERVER)?;
    let url = format!(
        "kryton://note/{}?account={}",
        urlencoding::encode(&note.note_path),
        note.account_id
    );
    let url_wide = to_wide(&url);
    let title_wide = to_wide(&note.title);
    link.SetPath(PCWSTR(url_wide.as_ptr()))?;
    link.SetDescription(PCWSTR(title_wide.as_ptr()))?;
    Ok(link)
}

/// Create an `IShellLinkW` for a named task with a given `kryton://` URL.
unsafe fn make_task_link(title: &str, url: &str) -> windows_core::Result<IShellLinkW> {
    let link: IShellLinkW =
        CoCreateInstance(&windows::Win32::UI::Shell::ShellLink, None, CLSCTX_INPROC_SERVER)?;
    let url_wide = to_wide(url);
    let title_wide = to_wide(title);
    link.SetPath(PCWSTR(url_wide.as_ptr()))?;
    link.SetDescription(PCWSTR(title_wide.as_ptr()))?;
    Ok(link)
}

/// Update the Windows taskbar jump list for the given account with up to 5 recent notes.
pub fn update_jumplist(
    account_id: &str,
    recent_notes: &[RecentNote],
) -> Result<(), String> {
    unsafe { update_jumplist_inner(account_id, recent_notes) }
        .map_err(|e| e.to_string())
}

unsafe fn update_jumplist_inner(
    account_id: &str,
    recent_notes: &[RecentNote],
) -> windows_core::Result<()> {
    let dest_list: ICustomDestinationList =
        CoCreateInstance(&DestinationList, None, CLSCTX_INPROC_SERVER)?;

    let mut min_slots: u32 = 0;
    let _removed: windows::Win32::UI::Shell::Common::IObjectArray =
        dest_list.BeginList(&mut min_slots)?;

    // --- Recent Notes category ---
    let notes_col: IObjectCollection =
        CoCreateInstance(&EnumerableObjectCollection, None, CLSCTX_INPROC_SERVER)?;
    for note in recent_notes.iter().take(5) {
        let link = make_note_link(note)?;
        notes_col.AddObject(&link)?;
    }
    if !recent_notes.is_empty() {
        let category_wide = to_wide("Recent Notes");
        let notes_arr: windows::Win32::UI::Shell::Common::IObjectArray =
            notes_col.cast()?;
        dest_list.AppendCategory(
            PCWSTR(category_wide.as_ptr()),
            &notes_arr,
        )?;
    }

    // --- Tasks: New Note, Open Quick Switcher ---
    let tasks_col: IObjectCollection =
        CoCreateInstance(&EnumerableObjectCollection, None, CLSCTX_INPROC_SERVER)?;

    let new_note_url = format!("kryton://new-note?account={}", account_id);
    let new_note_link = make_task_link("New Note", &new_note_url)?;
    tasks_col.AddObject(&new_note_link)?;

    let switcher_url = format!("kryton://quick-switcher?account={}", account_id);
    let switcher_link = make_task_link("Open Quick Switcher", &switcher_url)?;
    tasks_col.AddObject(&switcher_link)?;

    let tasks_arr: windows::Win32::UI::Shell::Common::IObjectArray = tasks_col.cast()?;
    dest_list.AddUserTasks(&tasks_arr)?;

    dest_list.CommitList()?;
    Ok(())
}
