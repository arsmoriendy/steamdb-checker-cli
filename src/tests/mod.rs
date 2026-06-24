use super::*;

macro_rules! state {
    (csv = $csv:expr, dir = $dir:expr) => {{
        let project_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests");
        let (csv, dir) = (
            project_dir.clone().join($csv),
            project_dir.clone().join($dir),
        );

        let entries = Entry::list(&csv).await.unwrap();

        Arc::new(Mutex::new(State {
            csv_path: csv.to_owned().into_os_string(),
            dir_path: dir.to_owned().into_os_string(),
            entries,
            ..Default::default()
        }))
    }};
}

macro_rules! check {
    ($state:expr) => {
        check($state.clone()).await.unwrap();

        loop {
            let state = $state.lock().await;
            let validation_progress = state.validation_progress;
            let validation_length = state.entries.len();
            if validation_progress == validation_length {
                break;
            }
            drop(state);
            sleep(Duration::from_millis(100)).await;
        }
    };
}

macro_rules! checked_state {
    (csv = $csv:expr, dir = $dir:expr) => {{
        let state = state!(csv = $csv, dir = $dir);

        check!(state);

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
