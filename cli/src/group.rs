use rusqlite::ToSql;

pub fn create(database: &rusqlite::Connection, name: &str) -> Result<(), anyhow::Error> {
    database.execute("INSERT INTO groups(id, name) VALUES (NULL, ?)", &[&name as &dyn ToSql])?;
    Ok(())
}

pub fn add_qubes<I>(database: &rusqlite::Connection, group_name: &str, qube_names: I) -> Result<(), anyhow::Error> where I: IntoIterator, I::Item: AsRef<str> {
    let mut empty = true;
    for qube_name in qube_names {
        empty = false;
        let qube_name: &str = qube_name.as_ref();

        database.execute("INSERT INTO vms(name, group_id) VALUES(?, (SELECT id FROM groups WHERE name = ?)) ON CONFLICT(name) DO UPDATE SET group_id = excluded.group_id", &[&qube_name as &dyn ToSql, &group_name as &dyn ToSql])?;
    }
    if empty {
        eprintln!("Warning: no qube specified, so nothing added");
    }
    Ok(())
}
