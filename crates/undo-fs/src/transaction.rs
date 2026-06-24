use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Operation;

// TODO: Add a disk-size cap for persisted history entries.
// `MAX_HISTORY_SIZE` limits the number of logical undo transactions,
// but a single transaction may contain many operations, e.g. moving a
// directory with thousands of files. Later, also enforce a maximum total
// storage size for `.undo/` and compact/delete old entries when exceeded.
const MAX_HISTORY_SIZE: usize = 100;

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub operations: Vec<Operation>,
}

pub struct TransactionHistory {
    pub history_size: usize,
    pub transactions: VecDeque<Transaction>,
    writer: BufWriter<File>,
    file_path: PathBuf,
}

impl TransactionHistory {
    pub fn new(history_file: impl AsRef<Path>) -> io::Result<Self> {
        let history_file = history_file.as_ref();

        if let Some(history_dir) = history_file.parent() {
            // Only attempt to create the dir if the path actually has a parent directory
            if !history_dir.as_os_str().is_empty() {
                fs::create_dir_all(history_dir)?;
            }
        }

        let transactions = Self::load_transactions(history_file)?;

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(history_file)?;

        Ok(Self {
            history_size: MAX_HISTORY_SIZE,
            transactions,
            writer: BufWriter::new(file),
            file_path: history_file.to_path_buf(), // Store it for compact_file()
        })
    }

    pub fn push(&mut self, transaction: Transaction) -> io::Result<()> {
        if self.transactions.len() >= self.history_size {
            let target_size = self.history_size / 2;

            while self.transactions.len() > target_size {
                self.transactions.pop_front();
            }

            // Because the file is append-only, removing old entries from memory
            // does not remove them from disk. Compact the file occasionally.
            self.compact_file()?;
        }

        let json = serde_json::to_string(&transaction).map_err(io::Error::other)?;

        writeln!(self.writer, "{json}")?;

        self.writer.flush()?;

        self.transactions.push_back(transaction);

        Ok(())
    }

    fn load_transactions(history_file: &Path) -> io::Result<VecDeque<Transaction>> {
        if !Path::new(history_file).exists() {
            return Ok(VecDeque::new());
        }

        let file = File::open(history_file)?;
        let reader = BufReader::new(file);

        let mut transactions = VecDeque::new();

        for line in reader.lines() {
            let line = line?;

            if line.trim().is_empty() {
                continue;
            }

            let transaction: Transaction = serde_json::from_str(&line).map_err(io::Error::other)?;

            transactions.push_back(transaction);
        }

        while transactions.len() > MAX_HISTORY_SIZE {
            transactions.pop_front();
        }

        Ok(transactions)
    }

    fn compact_file(&mut self) -> io::Result<()> {
        self.writer.flush()?;

        let temp_file = self.file_path.with_extension("tmp");

        {
            let file = File::create(&temp_file)?;
            let mut writer = BufWriter::new(file);

            for transaction in &self.transactions {
                let json = serde_json::to_string(transaction).map_err(io::Error::other)?;

                writeln!(writer, "{json}")?;
            }

            writer.flush()?;
        }

        fs::rename(temp_file, self.file_path.as_path())?;

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.file_path.as_path())?;

        self.writer = BufWriter::new(file);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Operation;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper to create a realistic dummy transaction using actual Operations
    fn dummy_transaction(id: usize) -> Transaction {
        let op = Operation::Move {
            from: PathBuf::from(format!("src_{}.txt", id)).into_boxed_path(),
            to: PathBuf::from(format!("dst_{}.txt", id)).into_boxed_path(),
            checksum: None,
        };

        Transaction {
            operations: vec![op],
        }
    }

    #[test]
    fn test_new_creates_directories_and_file() -> io::Result<()> {
        let dir = TempDir::new()?;

        // Use a nested path to ensure parent directories are created
        let file_path = dir.path().join("nested").join("history.jsonl");

        let _history = TransactionHistory::new(&file_path)?;

        assert!(file_path.exists(), "The history file should exist");
        assert!(
            file_path.parent().unwrap().exists(),
            "The parent directory should exist"
        );

        Ok(())
    }

    #[test]
    fn test_push_and_load_transactions() -> io::Result<()> {
        let dir = TempDir::new()?;
        let file_path = dir.path().join("history.jsonl");

        {
            let mut history = TransactionHistory::new(&file_path)?;
            history.push(dummy_transaction(1))?;
            history.push(dummy_transaction(2))?;
            assert_eq!(history.transactions.len(), 2);
        }

        // Reload history in a new instance and verify
        {
            let history = TransactionHistory::new(&file_path)?;
            assert_eq!(
                history.transactions.len(),
                2,
                "Transactions should be loaded correctly from disk"
            );

            // Verify the data inside the loaded transaction matches what we pushed
            #[allow(irrefutable_let_patterns)]
            if let Operation::Move { from, .. } = &history.transactions[0].operations[0] {
                assert_eq!(from.to_str().unwrap(), "src_1.txt");
            } else {
                panic!("Expected Operation::Move");
            }
        }

        Ok(())
    }

    #[test]
    fn test_history_compaction_at_capacity() -> io::Result<()> {
        let dir = TempDir::new()?;
        let file_path = dir.path().join("history.jsonl");

        let mut history = TransactionHistory::new(&file_path)?;

        // Push exactly MAX_HISTORY_SIZE transactions
        for i in 0..MAX_HISTORY_SIZE {
            history.push(dummy_transaction(i))?;
        }

        assert_eq!(history.transactions.len(), MAX_HISTORY_SIZE);

        // Pushing one more should trigger compaction (halving the list, then adding 1)
        history.push(dummy_transaction(MAX_HISTORY_SIZE))?;

        let expected_size = (MAX_HISTORY_SIZE / 2) + 1;
        assert_eq!(
            history.transactions.len(),
            expected_size,
            "History should be halved and include the newest transaction"
        );

        // Verify the persisted file state matches memory state
        let loaded_history = TransactionHistory::new(&file_path)?;
        assert_eq!(
            loaded_history.transactions.len(),
            expected_size,
            "Compacted disk state should exactly match the memory state"
        );

        Ok(())
    }

    #[test]
    fn test_ignores_empty_lines_on_load() -> io::Result<()> {
        let dir = TempDir::new()?;
        let file_path = dir.path().join("history.jsonl");

        // Manually write a corrupted/empty-line file
        {
            let mut file = File::create(&file_path)?;
            let tx = dummy_transaction(1);
            let json = serde_json::to_string(&tx)?;

            writeln!(file, "{}", json)?;
            writeln!(file, "   \n")?; // Empty and whitespace lines
            writeln!(file, "{}", json)?;
        }

        let history = TransactionHistory::new(&file_path)?;

        // Should successfully load the 2 valid transactions and safely ignore the empty lines
        assert_eq!(history.transactions.len(), 2);

        Ok(())
    }
}
