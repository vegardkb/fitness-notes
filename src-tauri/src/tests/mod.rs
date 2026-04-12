use crate::database::run_migrations;

pub mod scenario;

pub fn test_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    conn
}
