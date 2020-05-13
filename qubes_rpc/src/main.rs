use slog::{error, trace};
use anyhow::anyhow;
use std::io::BufRead;
use rusqlite::{ToSql, OptionalExtension};
use std::path::Path;
use sloggers::Build;

fn work(logger: &slog::Logger, data_dir: &Path) -> Result<(), anyhow::Error> {
    let mut db = ttt_common::db_connect(data_dir)?;
    let stdin = std::io::stdin();
    let stdin = stdin.lock();

    for line in stdin.lines() {
        let line = line?;
        let line = line.trim();
        let mut components = line.split(' ');
        let time = components.next().ok_or_else(|| anyhow!("Missing timestamp"))?.parse::<i64>()?;
        let qube = components.next().ok_or_else(|| anyhow!("Missing name of the VM"))?;
        trace!(logger, "Received switch event"; "qube" => qube, "time" => time);
        let tx = db.transaction()?;
        let prev_qube = tx
            .prepare("SELECT vm, begin FROM current_vm LIMIT 1")?
            .query_row(std::iter::empty::<&dyn ToSql>(),
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))).optional()?;
        tx.execute("REPLACE INTO current_vm VALUES(?, ?, 0);", &[&qube as &dyn ToSql, &time as &dyn ToSql])?;
        let (prev_qube, prev_begin) = match prev_qube {
            Some((vm, time)) => (vm, time),
            None => {
                tx.commit()?;
                continue;
            },
        };

        let prev_duration = time - prev_begin;
        // Duration can be 0 if windows are switched very fast
        if prev_duration > 0 {
            trace!(logger, "Inserting interval"; "prev_qube" => &prev_qube, "prev_begin" => prev_begin, "prev_duration" => prev_duration);
            tx
                .prepare("INSERT INTO intervals VALUES(?, ?, ?)")?
                .execute(&[&prev_qube as &dyn ToSql, &prev_begin, &prev_duration])?;
            tx.commit()?;
        }
    }

    Ok(())
}

fn create_file_logger(data_dir: &Path) -> Result<slog::Logger, anyhow::Error> {
    let logger = sloggers::file::FileLoggerBuilder::new(data_dir.join("qubes_rpc.log"))
        .level(sloggers::types::Severity::Trace)
        .build()?;
    Ok(logger)
}

fn create_term_logger() -> Result<slog::Logger, anyhow::Error> {
    let logger = sloggers::terminal::TerminalLoggerBuilder::new()
        .destination(sloggers::terminal::Destination::Stderr)
        .build()?;
    Ok(logger)
}

fn main() {
    use slog::o;
    let data_dir = ttt_common::default_data_dir().unwrap_or_else(|dd_err| {
        create_term_logger()
            .map(|logger| error!(logger, "failed to initialize data dir"; "error" => ?dd_err))
            .unwrap_or_else(|logger_err| {
                use std::io::Write;

                eprintln!("Failed to initialize data_dir: {:?}", dd_err);
                eprintln!("Failed to create a terminal logger: {:?}", logger_err);
                std::io::stderr().flush().expect("failed to flush stderr");
            });

        std::process::exit(99);
    });
    let logger = create_file_logger(&data_dir)
        .or_else(|err1| create_term_logger()
            .map(|logger| { error!(logger, "Failed to create file logger"; "error" => ?err1); logger })
            .map_err(move |err2| (err1, err2)))
        .unwrap_or_else(|(err1, err2)| {
            use std::io::Write;

            eprintln!("Failed to create file logger: {:?}", err1);
            eprintln!("Failed to create terminal logger: {:?}", err2);
            eprintln!("Future log messages will be discarded");
            std::io::stderr().flush().expect("failed to flush stderr");

            slog::Logger::root(slog::Discard, o!())
        });

    if let Err(err) = work(&logger, &data_dir) {
        error!(logger, "Trivial time tracker failed"; "error" => ?err);
        std::mem::drop(logger);
        std::process::exit(1);
    }
}
