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

    let state = State {
        csv_path: csv_path.to_owned().into_string().unwrap(),
        dir_path: dir_path.to_owned().into_string().unwrap(),
        entries,
        ..Default::default()
    };

    let state = Arc::new(Mutex::new(state));

    check(state.clone()).await?;

    loop {
        let state = state.lock().await;
        let validation_progress = state.validation_progress;
        let validation_length = state.entries.len();
        print!(
            "\rValidating {}/{} files",
            validation_progress, validation_length
        );
        if validation_progress == validation_length {
            break;
        }
        drop(state);
        sleep(Duration::from_millis(100)).await;
    }

    let state = Arc::try_unwrap(state).unwrap().into_inner();
    println!("{state}");

    Ok(())
}
