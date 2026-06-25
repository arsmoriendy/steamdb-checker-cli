mod checker;
mod entry;
mod find_extra;
mod prelude;
mod state;
mod verifier;

#[cfg(test)]
mod tests;

use checker::check;
use entry::Entry;
use prelude::*;
use state::State;

fn main() -> Result<()> {
    with_runtime(async { run().await.unwrap() });
    Ok(())
}

fn with_runtime(f: impl std::future::Future<Output = ()>) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(1024 * 1024 * 4) // 4 MiB
        .build()
        .unwrap()
        .block_on(f);
}

async fn run() -> Result<()> {
    let matches = command!()
        .arg(
            arg!(-c --csv <FILE> "Path of the csv file containing all relevant digests")
                .required(true),
        )
        .arg(arg!(-d --dir <DIR> "The root directory").required(true))
        .get_matches();

    let csv_path = Path::new(matches.get_one::<String>("csv").unwrap());
    let dir_path = Path::new(matches.get_one::<String>("dir").unwrap());

    let entries = Entry::list(csv_path).await?;
    let entries_n = entries.len();
    let (validation_tx, mut validation_rx) = mpsc::channel::<OsString>(entries_n);
    let (extra_tx, extra_rx) = oneshot::channel::<bool>();

    let state = State {
        csv_path: csv_path.to_owned().into_os_string(),
        dir_path: dir_path.to_owned().into_os_string(),
        entries,
        validation_tx: Some(validation_tx),
        extra_tx: Some(extra_tx),
        ..Default::default()
    };

    let state = Arc::new(Mutex::new(state));

    check(state.clone()).await?;

    let mut i = 0;
    while let Some(file) = validation_rx.recv().await {
        i += 1;
        println!("Validated {i}/{entries_n} entries: {file:?}");
        if i == entries_n {
            break;
        }
    }
    extra_rx.await?;

    let state = Arc::try_unwrap(state).unwrap().into_inner();
    println!("{state}");

    Ok(())
}
