/// Adapted from https://stackoverflow.com/a/45795699
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

trait KeyPair<Row, Col> {
    /// Obtains the row element of the pair.
    fn row(&self) -> &Row;
    /// Obtains the col element of the pair.
    fn col(&self) -> &Col;
}

impl<'a, Row, Col> Borrow<dyn KeyPair<Row, Col> + 'a> for (Row, Col)
where
    Row: Eq + Hash + 'a,
    Col: Eq + Hash + 'a,
{
    fn borrow(&self) -> &(dyn KeyPair<Row, Col> + 'a) {
        self
    }
}

impl<Row: Hash, Col: Hash> Hash for (dyn KeyPair<Row, Col> + '_) {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.row().hash(state);
        self.col().hash(state);
    }
}

impl<Row: Eq, Col: Eq> PartialEq for (dyn KeyPair<Row, Col> + '_) {
    fn eq(&self, other: &Self) -> bool {
        self.row() == other.row() && self.col() == other.col()
    }
}

impl<Row: Eq, Col: Eq> Eq for (dyn KeyPair<Row, Col> + '_) {}

pub struct Table<Row: Eq + Hash, Col: Eq + Hash, V> {
    map: HashMap<(Row, Col), V>,
}

impl<Row: Eq + Hash, Col: Eq + Hash, V> Table<Row, Col, V> {
    fn new() -> Self {
        Table {
            map: HashMap::new(),
        }
    }

    fn get(&self, row: &Row, col: &Col) -> Option<&V> {
        self.map.get(&(row, col) as &dyn KeyPair<Row, Col>)
    }

    fn set(&mut self, row: Row, col: Col, v: V) {
        self.map.insert((row, col), v);
    }
}

impl<Row, Col> KeyPair<Row, Col> for (Row, Col) {
    fn row(&self) -> &Row {
        &self.0
    }
    fn col(&self) -> &Col {
        &self.1
    }
}

impl<Row, Col> KeyPair<Row, Col> for (&Row, &Col) {
    fn row(&self) -> &Row {
        self.0
    }
    fn col(&self) -> &Col {
        self.1
    }
}
