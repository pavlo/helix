use helix_term::application::Application;
use helix_view::{doc, input::parse_macro};
use tokio_stream::wrappers::UnboundedReceiverStream;

#[cfg(windows)]
use crossterm::event::{Event, KeyEvent};
#[cfg(not(windows))]
use termina::event::{Event, KeyEvent};

use super::*;

async fn send_keys(app: &mut Application, keys: &str) -> anyhow::Result<()> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut rx_stream = UnboundedReceiverStream::new(rx);

    for key_event in parse_macro(keys)?.into_iter() {
        tx.send(Ok(Event::Key(KeyEvent::from(key_event))))?;
    }

    app.event_loop_until_idle(&mut rx_stream).await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn multi_root_file_picker_without_search_paths_reports_an_error() -> anyhow::Result<()> {
    let mut app = AppBuilder::new().build()?;

    send_keys(&mut app, "<space>m").await?;

    assert_eq!(
        app.editor.get_status().map(|(_, severity)| severity),
        Some(&helix_core::diagnostic::Severity::Error)
    );

    send_keys(&mut app, "<esc>").await?;
    let errs = app.close().await;
    assert!(errs.is_empty(), "errors closing app: {errs:?}");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn multi_root_file_picker_searches_displayed_paths_and_opens_the_absolute_path(
) -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let first = root.path().join("root_alpha");
    let second = root.path().join("root_beta");

    let duplicate = first.join("nested").join("unique_two.txt");
    std::fs::create_dir_all(duplicate.parent().unwrap())?;
    std::fs::write(&duplicate, "first root")?;

    let target = second.join("nested").join("unique_two.txt");
    std::fs::create_dir_all(target.parent().unwrap())?;
    std::fs::write(&target, "second root")?;

    let mut config = test_config();
    config.editor.file_picker.search_paths = vec![first, second];

    let mut app = AppBuilder::new().with_config(config).build()?;

    send_keys(&mut app, "<space>m").await?;
    // The root name and the file name are both part of the displayed path, so native
    // multi-term matching can disambiguate identical relative paths in different roots.
    send_keys(&mut app, "root_beta unique_two").await?;
    send_keys(&mut app, "<ret>").await?;

    assert_eq!(doc!(app.editor).path(), Some(target.as_path()));

    let errs = app.close().await;
    assert!(errs.is_empty(), "errors closing app: {errs:?}");

    Ok(())
}
