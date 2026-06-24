use crate::{prelude::*, state::State};
use std::io::ErrorKind::NotFound;
use tokio::io::AsyncReadExt;

pub async fn verify_entries(state: Arc<Mutex<State>>) -> Result<()> {
    let mtx = state.lock().await;
    let entries = mtx.entries.clone();
    let dir_str = mtx.dir_path.clone();
    let dir = Path::new(&dir_str);
    drop(mtx);

    for (path, truncated_hash) in entries {
        let path = dir.join(path);
        let Some(truncated_hash) = truncated_hash.clone() else {
            let mut mtx = state.lock().await;
            mtx.validation_progress += 1;
            drop(mtx);
            continue;
        };

        let state = state.clone();
        spawn(async move {
            let status = verify_file(path.as_path(), truncated_hash.as_str())
                .await
                .unwrap();

            let mut mtx = state.lock().await;

            use EntryStatus::*;
            match status {
                Valid => &mut mtx.valid,
                Missing => &mut mtx.missing,
                Invalid => &mut mtx.invalid,
            }
            .push(path.into_string().unwrap());

            mtx.validation_progress += 1;
        });
    }
    Ok(())
}

pub enum EntryStatus {
    Valid,
    Missing,
    Invalid,
}

pub async fn verify_file(path: &Path, truncated_hash: &str) -> Result<EntryStatus> {
    let mut file = match File::open(path).await {
        Ok(f) => f,
        Err(e) => {
            match e.kind() {
                NotFound => return Ok(EntryStatus::Missing),
                _ => return Err(e.into()),
            };
        }
    };

    const BUF_SIZE: usize = 1_048_576;
    let mut buf = [0; BUF_SIZE];
    let mut hash = Sha1::new();
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    let hash = hash.finalize();

    let real_hash = hex::encode(hash);
    let real_truncated_hash = format!("{}***{}", &real_hash[..10], &real_hash[30..]);
    if truncated_hash == &real_truncated_hash {
        return Ok(EntryStatus::Valid);
    }

    Ok(EntryStatus::Invalid)
}
