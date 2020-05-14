mod stats;
use chrono::Local;

fn process_stats(mut args: std::env::Args) -> Result<(), anyhow::Error> {
    use chrono::Datelike;

    let period0 = args.next().ok_or_else(|| anyhow::anyhow!("udefined period, known periods: today, yesterday, this {week,month,year}, 'start datetime' {day,week,month,year}, 'start datetime' 'end datetime'"))?;
    let (begin, end): (_, Option<chrono::DateTime<Local>>) = match &*period0 {
        "today" => (Local::today().and_time(chrono::NaiveTime::from_num_seconds_from_midnight(0, 0)).expect("invalid datetime"), None),
        "yesterday" => (Local::today().pred().and_time(chrono::NaiveTime::from_num_seconds_from_midnight(0, 0)).expect("invalid datetime"), Some(Local::today().and_time(chrono::NaiveTime::from_num_seconds_from_midnight(0, 0)).expect("invalid datetime"))),
        "this" => {
            let period0_0 = args.next().ok_or_else(|| anyhow::anyhow!("missing either week, month, or year"))?;
            (match &*period0_0 {
                "week" => {
                    let mut day = Local::today();
                    // TODO: some cultures count days from Sunday, this should be determined from
                    // locale
                    let week_start_day = chrono::Weekday::Mon;
                    while day.weekday() != week_start_day {
                        day = day.pred();
                    }
                    day
                },
                "month" => Local::today().with_day0(0).expect("invalid day of month"),
                "year" => Local::today().with_ordinal(1).expect("invalid day of year"),
                unknown => anyhow::bail!("unknown period {}, known periods: week, month, year", unknown),
            }.and_time(chrono::NaiveTime::from_num_seconds_from_midnight(0, 0)).expect("invalid datetime"), None)
        },
        datetime => (datetime.parse()?, args.next().map(|end| end.parse()).transpose()?),
    };

    let interval = stats::Interval {
        from: begin.into(),
        to: end.map(Into::into),
    };

    let data_dir = ttt_common::default_data_dir()?;
    let db = ttt_common::db_connect(data_dir)?;

    stats::fetch_stats(&db, interval, None)
}

fn main() -> Result<(), anyhow::Error> {
    let mut args = std::env::args();
    let _program = args.next().ok_or_else(|| anyhow::anyhow!("not even zeroth argument present"))?;
    let command = args.next().ok_or_else(|| anyhow::anyhow!("no command given, available commands: stats"))?;

    match &*command {
        "stats" => process_stats(args)?,
        unknown => anyhow::bail!("unknown command: {}", unknown),
    }
    Ok(())
}
