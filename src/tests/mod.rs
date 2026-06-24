use super::*;

#[test]
fn valid() {
    with_runtime(async {
        let project_dir = env!("CARGO_MANIFEST_DIR");
        let (csv, dir) = (
            Path::new(project_dir).join("src/tests/assets/dir1_sum.csv"),
            Path::new(project_dir).join("src/tests/assets/dir1"),
        );

        let entries = Entry::list(&csv).await.unwrap();

        let state = State {
            csv_path: csv.to_owned().into_string().unwrap(),
            dir_path: dir.to_owned().into_string().unwrap(),
            entries,
            ..Default::default()
        };

        let state = Arc::new(Mutex::new(state));

        check(state.clone()).await.unwrap();

        loop {
            let state = state.lock().await;
            let validation_progress = state.validation_progress;
            let validation_length = state.entries.len();
            if validation_progress == validation_length {
                break;
            }
            drop(state);
            sleep(Duration::from_millis(100)).await;
        }

        let state = Arc::try_unwrap(state).unwrap().into_inner();

        assert_eq!(state.extra.len(), 0);
        assert_eq!(state.missing.len(), 0);
        assert_eq!(state.invalid.len(), 0);
        assert_eq!(state.valid.len(), 17);
    });
}
