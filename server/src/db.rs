use std::io;
use std::sync::Arc;
use std::fs;
use dashmap::DashMap;
use chrono::{naive::NaiveDateTime, DateTime, Local};
use serde::{Deserialize, Serialize};
use derive_more::{Deref, DerefMut};


pub const DATE_FORMAT: &str = "%Y-%m-%d H%H";

type LocalTime = NaiveDateTime;
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

    pub async fn list(&self, args: &[&str]) -> Vec<String> {
        let mut list = Vec::new();

        let filter = self.parse_args(args);

        for set in &self.0 {
            let (key, val) = set.pair();
            
            list.push(format!("{}: {}", key, val));
        }

        list
    }

    async fn parse_args(&self, args: &[&str]) -> OutputFilter {
        if self.is_filtered(args) == false {
            return OutputFilter::builder().build();
        }

        let filter = OutputFilter::builder();

        for i in 0..args.len() {
            let arg = &args[i];
            match arg {
                "<" => filter.add_filter(&Comp::Lesser, )
            }
        }

        filter.build()
    }

    async fn is_filtered(&self, args: &[&str]) -> bool {
        for arg in args {
            if arg.contains("if") {
                return true;
            }
        }
        false
    }

    async fn string_to_date(string: &str) -> LocalTime {
        DateTime::parse_from_str(string, DATE_FORMAT).unwrap().naive_local()
    }

    async fn build_times(arg1: &str, arg2: &str) -> Option<LocalTime> {
        match arg1 {
            "year" => LocalTime::parse_from_str(&format!("{}-1-1 H0", arg2), DATE_FORMAT).ok(),
            "month" => LocalTime::parse_from_str(&format!("1-{}-1 H0", arg2), DATE_FORMAT).ok(),
            "day" => LocalTime::parse_from_str(&format!("{}-1-1 H0", arg2), DATE_FORMAT).ok(),
            "hour" => LocalTime::parse_from_str(&format!("{}-1-1 H0", arg2), DATE_FORMAT).ok(),
            "price" => LocalTime::parse_from_str(&format!("{}-1-1 H0", arg2), DATE_FORMAT).ok(),
            _ => None,
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

    fn filter(&self, ) -> Vec<String> {
        let mut filtered_list = Vec::new();

        for set in db.iter() {
            let (key, val) = set.pair();
            
            if let Some(string) = self.filter_pair(key, val) {
                filtered_list.push(string);
            }
        }       

        filtered_list
    }

    fn filter_pair(&self, key: &DbKey, val: &DbVal) -> Option<String> {
        let mut filters_failed = 0;
        let dt = DateTime::parse_from_str(key, DATE_FORMAT).unwrap().naive_local();

        filters_failed += match &self.year {
            Some(f) => if (*f)(&dt) { 0 } else { 1 },
            None => 0
        };

        filters_failed += match &self.month {
            Some(f) => if (*f)(&dt) { 0 } else { 1 },
            None => 0
        };

        filters_failed += match &self.day {
            Some(f) => if (*f)(&dt) { 0 } else { 1 },
            None => 0
        };

        filters_failed += match &self.hour {
            Some(f) => if (*f)(&dt) { 0 } else { 1 },
            None => 0
        };

        filters_failed += match &self.val {
            Some(f) => if (*f)(&val) { 0 } else { 1 },
            None => 0
        };
    
        if filters_failed == 0 {
            Some(format!("{}: {}", key, val))
        } else {
            None
        }
    }
}


impl OutputFilterBuilder {
    async fn add_filter(self, comp: &Comp, filter_type: FilterType) -> OutputFilterBuilder {
        match filter_type {
            FilterType::Year(y) => self.year(comp, y),
            FilterType::Month(m) => self.month(comp, m),
            FilterType::Day(d) => self.day(comp, d),
            FilterType::Hour(h) => self.hour(comp, h),
            FilterType::Price(p) => self.val(comp, p)
        }
    }

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

enum FilterType {
    Year(LocalTime),
    Month(LocalTime),
    Day(LocalTime),
    Hour(LocalTime),
    Price(f64)
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
