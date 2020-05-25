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

fn print_table(item_header: &str, duration_header: &str, items: &[(String, chrono::Duration)]) {
    use std::cmp::max;

    let max_item_len = items.iter().fold(item_header.len(),
        |max_item, &(ref item, _)| max(max_item, item.len()));

    print!("{}", item_header);
    for _ in 0..(max_item_len - item_header.len()) {
        print!(" ");
    }
    print!(" | ");
    println!("{}", duration_header);

    for &(ref item, ref duration) in items {
        print!("{}", item);
        for _ in 0..(max_item_len - item.len()) {
            print!(" ");
        }
        print!(" | ");
        // Duration has a retarded Display impl
        let hours = duration.num_hours();
        let minutes = duration.num_minutes() - hours * 60;
        let seconds = duration.num_seconds() - hours * 3600 - minutes * 60;
        println!("{}:{}:{}", hours, minutes, seconds);
    }
}

pub fn fetch_stats(database: &rusqlite::Connection, interval: Interval, vm: Option<&str>) -> Result<(), anyhow::Error> {
    let mut stmt;
    let vm_stats = match (&interval.to, &vm) {
        (None, None) => {
            stmt = database.prepare("SELECT vm, SUM(duration) FROM intervals LEFT OUTER JOIN vms ON intervals.vm = vms.name WHERE intervals.begin >= ? AND vms.group_id IS NULL GROUP BY intervals.vm ORDER BY intervals.vm ASC")?;
            stmt.query(&[&interval.from.0 as &dyn ToSql])?
        },
        (None, Some(vm)) => {
            stmt = database.prepare("SELECT vm, SUM(duration) FROM intervals LEFT OUTER JOIN vms ON intervals.vm = vms.name WHERE intervals.begin >= ? AND intervals.vm = ? AND vms.group_id IS NULL")?;
            stmt.query(&[&interval.from.0 as &dyn ToSql, &*vm as &dyn ToSql])?
        },
        (Some(to), None) => {
            stmt = database.prepare("SELECT vm, SUM(duration) FROM intervals LEFT OUTER JOIN vms ON intervals.vm = vms.name WHERE intervals.begin >= ? AND intervals.begin + intervals.duration < ? AND vms.group_id IS NULL GROUP BY intervals.vm ORDER BY intervals.vm ASC")?;
            stmt.query(&[&interval.from.0 as &dyn ToSql, &to.0 as &dyn ToSql])?
        },
        (Some(to), Some(vm)) => {
            stmt = database.prepare("SELECT vm, SUM(duration) FROM intervals LEFT OUTER JOIN vms ON intervals.vm = vms.name WHERE intervals.begin >= ? AND intervals.vm = ? AND intervals.begin + intervals.duration < ? AND vms.group_id IS NULL")?;
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

    let group_stats = match (&interval.to, &vm) {
        (None, None) => {
            stmt = database.prepare("SELECT groups.name, SUM(duration) FROM intervals INNER JOIN vms ON intervals.vm = vms.name INNER JOIN groups ON vms.group_id = groups.id WHERE intervals.begin >= ? GROUP BY groups.name ORDER BY groups.name ASC")?;
            stmt.query(&[&interval.from.0 as &dyn ToSql])?
        },
        (None, Some(vm)) => {
            stmt = database.prepare("SELECT vm, SUM(duration) FROM intervals INNER JOIN vms ON intervals.vm = vms.name INNER JOIN groups ON vms.group_id = groups.id WHERE intervals.begin >= ? AND intervals.vm = ?")?;
            stmt.query(&[&interval.from.0 as &dyn ToSql, &*vm as &dyn ToSql])?
        },
        (Some(to), None) => {
            stmt = database.prepare("SELECT vm, SUM(duration) FROM intervals INNER JOIN vms ON intervals.vm = vms.name INNER JOIN groups ON vms.group_id = groups.id WHERE intervals.begin >= ? AND intervals.begin + intervals.duration < ? GROUP BY intervals.vm ORDER BY intervals.vm ASC")?;
            stmt.query(&[&interval.from.0 as &dyn ToSql, &to.0 as &dyn ToSql])?
        },
        (Some(to), Some(vm)) => {
            stmt = database.prepare("SELECT vm, SUM(duration) FROM intervals INNER JOIN vms ON intervals.vm = vms.name INNER JOIN groups ON vms.group_id = groups.id WHERE intervals.begin >= ? AND intervals.vm = ? AND intervals.begin + intervals.duration < ?")?;
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

    print_table("Group", "Time spent", &group_stats);
    println!();
    print_table("Qube", "Time spent", &vm_stats);

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
