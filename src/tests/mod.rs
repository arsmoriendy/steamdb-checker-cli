use super::*;

macro_rules! checked_state {
    (csv = $csv:expr, dir = $dir:expr) => {{
        let project_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests");
        let (csv_path, dir_path) = (
            project_dir.clone().join($csv),
            project_dir.clone().join($dir),
        );

        let entries = Entry::list(&csv_path).await.unwrap();
        let entries_n = entries.len();
        let (validation_tx, mut validation_rx) = mpsc::channel::<OsString>(entries_n);
        let (extra_tx, extra_rx) = oneshot::channel::<bool>();

        let state = Arc::new(Mutex::new(State {
            csv_path: csv_path.to_owned().into_os_string(),
            dir_path: dir_path.to_owned().into_os_string(),
            entries,
            validation_tx: Some(validation_tx),
            extra_tx: Some(extra_tx),
            ..Default::default()
        }));

        check(state.clone()).await.unwrap();

        let mut i = 0;
        while let Some(file) = validation_rx.recv().await {
            i += 1;
            println!("Validated {i}/{entries_n} entries: {file:?}");
            if i == entries_n {
                break;
            }
        }
        extra_rx.await.unwrap();

        Arc::try_unwrap(state).unwrap().into_inner()
    }};
}

#[test]
fn valid() {
    with_runtime(async {
        let state = checked_state!(csv = "./assets/dir1_sum.csv", dir = "./assets/dir1/");

        assert_eq!(state.extra.len(), 0);
        assert_eq!(state.missing.len(), 0);
        assert_eq!(state.invalid.len(), 0);
        assert_eq!(state.valid.len(), 17);
    });
}

#[test]
fn extra() {
    with_runtime(async {
        let state = checked_state!(csv = "./assets/dir1_extra_sum.csv", dir = "./assets/dir1/");

        assert_eq!(state.extra.len(), 3);
        assert_eq!(state.missing.len(), 0);
        assert_eq!(state.invalid.len(), 0);
        assert_eq!(state.valid.len(), 17 - 3);
    });
}

#[test]
fn invalid() {
    with_runtime(async {
        let state = checked_state!(
            csv = "./assets/dir1_invalid_sum.csv",
            dir = "./assets/dir1/"
        );

        assert_eq!(state.extra.len(), 0);
        assert_eq!(state.missing.len(), 0);
        assert_eq!(state.invalid.len(), 3);
        assert_eq!(state.valid.len(), 17 - 3);
    });
}

#[test]
fn missing() {
    with_runtime(async {
        let state = checked_state!(
            csv = "./assets/dir1_missing_sum.csv",
            dir = "./assets/dir1/"
        );

        assert_eq!(state.extra.len(), 0);
        assert_eq!(state.missing.len(), 2);
        assert_eq!(state.invalid.len(), 0);
        assert_eq!(state.valid.len(), 19 - 2);
    });
}
