mod entry;
mod find_extra;
mod prelude;
mod state;
mod verifier;

use entry::Entry;
use find_extra::find_extra;
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

    let state = State {
        csv_path: csv_path.to_owned().into_string().unwrap(),
        dir_path: dir_path.to_owned().into_string().unwrap(),
        entries,
        ..Default::default()
    };

    let state = Arc::new(Mutex::new(state));

    verify_entries(dir_path, state.clone()).await?;

    let state2 = state.clone();
    let dir_path2 = dir_path.to_owned();
    spawn(async move {
        find_extra(&dir_path2, state2).await.unwrap();
    });

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
