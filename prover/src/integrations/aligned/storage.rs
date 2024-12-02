use rusqlite::{params, Connection};

pub struct ProofDB {
    pub path: String,
}

impl ProofDB {
    pub fn setup(&self) {
        let conn = Connection::open(&self.path).expect("Failed to connect to db");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS proofs (
                      root    BLOB PRIMARY KEY,
                      height   INTEGER NOT NULL
                      )",
            [],
        )
        .expect("Failed to create table");
    }

    pub fn insert(&mut self, root: &[u8], height: usize) {
        let conn = Connection::open(&self.path).expect("Failed to connect to db");
        conn.execute(
            "INSERT OR REPLACE INTO proofs (root, height) VALUES (?1, ?2)",
            params![root, height],
        )
        .expect("Failed to insert");
    }

    pub fn get_all(&self) -> Vec<(Vec<u8>, i32)> {
        let conn = Connection::open(&self.path).expect("Failed to connect to db");
        let mut stmt = conn
            .prepare("SELECT root, height FROM proofs")
            .expect("Failed to connect");
        let node_iter = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("Failed to query");

        let mut proofs = Vec::new();
        for node in node_iter {
            proofs.push(node.expect("Failed to get proofs"));
        }
        proofs
    }
}
