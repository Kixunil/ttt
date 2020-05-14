use rusqlite::ToSql;

pub struct Timestamp(i64);

impl<T> From<chrono::DateTime<T>> for Timestamp where T: chrono::TimeZone {
    fn from(value: chrono::DateTime<T>) -> Self {
        Timestamp(value.timestamp())
    }
}

pub struct Interval {
    pub from: Timestamp,
    pub to: Option<Timestamp>,
}

pub fn fetch_stats(database: &rusqlite::Connection, interval: Interval, vm: Option<&str>) -> Result<(), anyhow::Error> {
    use std::cmp::max;

    let mut stmt;
    let stats = match (&interval.to, &vm) {
        (None, None) => {
            stmt = database.prepare("SELECT vm, SUM(duration) FROM intervals WHERE begin >= ? GROUP BY vm ORDER BY vm ASC")?;
            stmt.query(&[&interval.from.0 as &dyn ToSql])?
        },
        (None, Some(vm)) => {
            stmt = database.prepare("SELECT vm, SUM(duration) FROM intervals WHERE begin >= ? AND vm = ?")?;
            stmt.query(&[&interval.from.0 as &dyn ToSql, &*vm as &dyn ToSql])?
        },
        (Some(to), None) => {
            stmt = database.prepare("SELECT vm, SUM(duration) FROM intervals WHERE begin >= ? AND begin + duration < ? GROUP BY vm ORDER BY vm ASC")?;
            stmt.query(&[&interval.from.0 as &dyn ToSql, &to.0 as &dyn ToSql])?
        },
        (Some(to), Some(vm)) => {
            stmt = database.prepare("SELECT vm, SUM(duration) FROM intervals WHERE begin >= ? AND vm = ? AND begin + duration < ?")?;
            stmt.query(&[&interval.from.0 as &dyn ToSql, &to.0 as &dyn ToSql, &*vm as &dyn ToSql])?
        },
    }
    .mapped(|row| {
        let vm = row.get::<_, String>(0)?;
        let sum = row.get::<_, i64>(1)?;
        let sum = chrono::Duration::seconds(sum);
        Ok((vm, sum))
    })
    // We need to collect in order to be able to create headers
    .collect::<Result<Vec<_>, _>>()?;

    let vm_header = "Qube";
    let duration_header = "Time spent";
    let max_vm = stats.iter().fold(vm_header.len(),
        |max_vm, &(ref vm, _)| max(max_vm, vm.len()));

    print!("{}", vm_header);
    for _ in 0..(max_vm - vm_header.len()) {
        print!(" ");
    }
    print!(" | ");
    println!("{}", duration_header);

    for &(ref vm, ref duration) in &stats {
        print!("{}", vm);
        for _ in 0..(max_vm - vm.len()) {
            print!(" ");
        }
        print!(" | ");
        // Duration has a retarded Display impl
        let hours = duration.num_hours();
        let minutes = duration.num_minutes() - hours * 60;
        let seconds = duration.num_seconds() - hours * 3600 - minutes * 60;
        println!("{}:{}:{}", hours, minutes, seconds);
    }

    Ok(())
}

/*
fn display_len<T: std::fmt::Display>(val: &T) -> usize {
    struct Sum(usize);

    impl std::fmt::Write for Sum {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            self.0 += s.len();
        }

        fn write_char(&mut self, c: char) -> std::fmt::Result {
            self.0 += c.utf8_len();
        }
    }

    Sum(0).write_fmt(format_args!("{}", val)).expect("Sum never fails")
}
*/
