use crate::{prelude::*, state::State};

pub async fn find_extra(dir: &Path, state: Arc<Mutex<State>>) -> Result<()> {
    let mut root_dir = read_dir(dir).await?;
    loop {
        let Some(entry) = root_dir.next_entry().await? else {
            break;
        };

        let path = entry.path();
        if path.is_dir() {
            Box::pin(find_extra(&path, state.clone())).await?;
            continue;
        }

        let mut mtx = state.lock().await;
        let truncated_path = path.strip_prefix(Path::new(&mtx.dir_path))?;
        let is_extra = mtx.entries.get(truncated_path).is_none();
        if is_extra {
            let path_str = path.into_string().unwrap();
            mtx.extra.push(path_str);
        }
        drop(mtx)
    }
    Ok(())
}
