use crate::{find_extra::find_extra, prelude::*, state::State, verifier::verify_entries};

pub async fn check(state: Arc<Mutex<State>>) -> Result<()> {
    verify_entries(state.clone()).await?;

    let state = state.clone();

    let mtx = state.lock().await;
    let root_str = mtx.dir_path.clone();
    let root = Path::new(&root_str).to_owned();
    drop(mtx);

    spawn(async move {
        find_extra(&root, state.clone()).await.unwrap();
        let mut mtx = state.lock().await;
        mtx.checked_extras = true;
        drop(mtx);
    });

    Ok(())
}
