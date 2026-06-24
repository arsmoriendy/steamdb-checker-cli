mod entry;
mod prelude;
mod state;
mod verifier;

use entry::Entry;
use prelude::*;
use state::State;
use verifier::verify_entries;

fn main() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(4194304) // 4 MiB
        .build()?
        .block_on(async { run().await.unwrap() });
    Ok(())
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

    let mut state = State::default();
    state.validation_length = entries.len();

    let state = Arc::new(Mutex::new(state));

    verify_entries(&entries, dir_path, state.clone()).await?;

    loop {
        let state = state.lock().await;
        print!(
            "\rValidating {}/{} files",
            state.validation_progress, state.validation_length
        );
        if state.validation_progress == state.validation_length {
            break;
        }
        drop(state);
        sleep(Duration::from_millis(100)).await;
    }

    println!("{state:?}");

    Ok(())
}
