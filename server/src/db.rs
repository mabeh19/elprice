use std::io;
use std::sync::Arc;
use std::fs;
use dashmap::DashMap;
use chrono::{naive::NaiveDate, DateTime, Local};
use serde::{Deserialize, Serialize};
use derive_more::{Deref, DerefMut};

type LocalTime = DateTime<Local>;
type FilterTimeCallback = Option<Box<dyn Fn(&LocalTime) -> bool>>;
type FilterValueCallback = Option<Box<dyn Fn(&f64) -> bool>>;
type DbKey = String;
type DbVal = f64;
type DbInternal = DashMap<DbKey, DbVal>;
pub type Database = Arc<Db>; 

#[derive(Clone, Serialize, Deserialize, Deref, DerefMut)]
pub struct Db(DbInternal);

impl Db {
    pub fn new() -> Self {
        Self {
            0: DashMap::new()
        }
    }

    pub async fn get(&self, key: &DbKey) -> Option<DbVal> {
        if let Some(val) = self.0.get(key) {
            return Some(*val);
        }
        return None;
    }

    pub async fn insert(&self, key: DbKey, val: DbVal) {
        self.0.insert(key, val);
    }

    pub async fn save(&self) {
    const DB_FILE: &str = ".db.json";
    const BACKUP_FILE: &str = ".db.json.backup";
        let map_as_json = serde_json::to_string(&self.0).unwrap();
//        println!("Serialized: {}", map_as_json);
        fs::write(DB_FILE, map_as_json.clone());
        fs::write(BACKUP_FILE, map_as_json);
    }

    pub async fn load(&mut self) {
    const DB_FILE: &str = ".db.json";
        if let Ok(db_as_string) = fs::read_to_string(DB_FILE) {
            let map_as_json: DbInternal = serde_json::from_str(&db_as_string).unwrap();
            self.0 = map_as_json;
        }
    }

    pub async fn list(&self, args: &[&str]) {
        for arg in args {

        }
    }
}

struct OutputFilter {
    year:  FilterTimeCallback,
    month: FilterTimeCallback,
    day:   FilterTimeCallback,
    hour:  FilterTimeCallback, 
    val: FilterValueCallback
}

struct OutputFilterBuilder {
    year:  FilterTimeCallback,
    month: FilterTimeCallback,
    day:   FilterTimeCallback,
    hour:  FilterTimeCallback,
    val: FilterValueCallback
}

impl OutputFilter {
    fn builder() -> OutputFilterBuilder {
        OutputFilterBuilder {
            year:   None,
            month:  None,
            day:    None,
            hour:   None,
            val:    None
        }
    }

    fn filter(&self, key: &DbKey, val: &DbVal) {

    }
}


impl OutputFilterBuilder {
    fn year(mut self, comp: &Comp, yr: LocalTime) -> OutputFilterBuilder {
        self.year = Self::construct_time_filter(comp, yr);
        self
    }

    fn month(mut self, comp: &Comp, mnth: LocalTime) -> OutputFilterBuilder {
        self.month = Self::construct_time_filter(comp, mnth);
        self
    }

    fn day(mut self, comp: &Comp, day: LocalTime) -> OutputFilterBuilder {
        self.day = Self::construct_time_filter(comp, day);
        self
    }

    fn hour(mut self, comp: &Comp, hour: LocalTime) -> OutputFilterBuilder {
        self.hour = Self::construct_time_filter(comp, hour);
        self
    }

    fn val(mut self, comp: &Comp, val: f64) -> OutputFilterBuilder {
        self.val = Self::construct_val_filter(comp, val);
        self
    }

    fn build(self) -> OutputFilter {
        OutputFilter {
            year:   self.year,
            month:  self.month,
            day:    self.day,
            hour:   self.hour,
            val:    self.val
        }
    }

    fn construct_time_filter(comp: &Comp, time: LocalTime) -> FilterTimeCallback {
        Some(match comp {
            Comp::Lesser        => Box::new(move |x: &LocalTime| { *x <  time }),
            Comp::LesserEqual   => Box::new(move |x: &LocalTime| { *x <= time }),
            Comp::Equal         => Box::new(move |x: &LocalTime| { *x == time }),
            Comp::GreaterEqual  => Box::new(move |x: &LocalTime| { *x >= time }),
            Comp::Greater       => Box::new(move |x: &LocalTime| { *x >  time }),
        })
    }

    fn construct_val_filter(comp: &Comp, val: f64) -> FilterValueCallback {
        Some(match comp {
            Comp::Lesser        => Box::new(move |x: &f64| { *x <  val }),
            Comp::LesserEqual   => Box::new(move |x: &f64| { *x <= val }),
            Comp::Equal         => Box::new(move |x: &f64| { *x == val }),
            Comp::GreaterEqual  => Box::new(move |x: &f64| { *x >= val }),
            Comp::Greater       => Box::new(move |x: &f64| { *x >  val }),
        })
    }
}

enum Comp {
    Lesser,
    LesserEqual,
    Equal,
    GreaterEqual,
    Greater
}

#[tokio::test]
async fn save_load_test() {
    let key = "1234".to_string();
    let database = Arc::new(Db::new());
    database.insert(key.clone(), 25.).await;
    database.save().await;
    let orig_val = database.get(&key).await;

    let mut new_db = Db::new();
    new_db.load().await;
    let new_db = Arc::new(new_db);
    let clone_val = new_db.get(&key).await;

    println!("Orig_val: {:?}, clone_val: {:?}", orig_val, clone_val);
    assert!(orig_val == clone_val);
}
