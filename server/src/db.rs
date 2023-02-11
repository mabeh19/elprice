use std::sync::Arc;
use std::fs;
use dashmap::DashMap;
use chrono::{naive::NaiveDateTime, DateTime};
use serde::{Deserialize, Serialize};
use derive_more::{Deref, DerefMut};

const DB_FILE: &str = ".db.json";
const BACKUP_FILE: &str = ".db.json.backup";

pub const DATE_FORMAT: &str = "%Y-%m-%d H%H";

type LocalTime = NaiveDateTime;
type FilterCallback<T> = Option<fn(a: T, b: T) -> bool>;
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
        let map_as_json = serde_json::to_string(&self.0).unwrap();
//        println!("Serialized: {}", map_as_json);
        if let Err(e) = fs::write(DB_FILE, map_as_json.clone()) {
            println!("Failed main save: {}", e);
        }
        if let Err(e) = fs::write(BACKUP_FILE, map_as_json) {
            println!("Failed to save to backup: {}", e);
        }
    }

    pub async fn load(&mut self) {
        if let Ok(db_as_string) = fs::read_to_string(DB_FILE) {
            let map_as_json: DbInternal = serde_json::from_str(&db_as_string).unwrap();
            self.0 = map_as_json;
        }
    }

    pub async fn list(&self, args: &[&str]) -> Vec<String> {

        let filter = self.parse_args(args).await;
        let res = filter.filter(self.0.clone());
  
        res.await
    }

    async fn parse_args(&self, args: &[&str]) -> OutputFilter {
        if self.is_filtered(args).await == false {
            return OutputFilter::builder().build();
        }

        let mut filter = OutputFilter::builder();

        for i in 0..args.len() {
            let arg = args[i];
            let comp = Comp::from_str(arg); 
            let f_type = args[i - 1];
            let val = args[i + 1];
            let time = Self::build_times(f_type, val).await;

            if comp.is_some() {
                let f_type = match f_type.to_lowercase().as_str() {
                    "year" | "month" | "day" | "hour" => FilterType::new_time(f_type, time.unwrap()), 
                    "price" => FilterType::new_price(f_type, val.parse().unwrap()),
                    _       => None
                };

                if let Some(f_type) = f_type {
                    filter = filter.add_filter(comp.unwrap(), f_type).await;
                }
            }
        }

        filter.build()
    }

    async fn is_filtered(&self, args: &[&str]) -> bool {
        for arg in args {
            if arg.contains("where") || arg.contains("if") {
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
            "day" => LocalTime::parse_from_str(&format!("1-1-{} H0", arg2), DATE_FORMAT).ok(),
            "hour" => LocalTime::parse_from_str(&format!("1-1-1 H{}", arg2), DATE_FORMAT).ok(),
            _ => None,
        }
    }
}

struct OutputFilter {
    year:  (FilterCallback<LocalTime>, Option<LocalTime>),
    month: (FilterCallback<LocalTime>, Option<LocalTime>),
    day:   (FilterCallback<LocalTime>, Option<LocalTime>),
    hour:  (FilterCallback<LocalTime>, Option<LocalTime>),
    val: (FilterCallback<f64>, f64)
}

struct OutputFilterBuilder {
    year:  (FilterCallback<LocalTime>, Option<LocalTime>),
    month: (FilterCallback<LocalTime>, Option<LocalTime>),
    day:   (FilterCallback<LocalTime>, Option<LocalTime>),
    hour:  (FilterCallback<LocalTime>, Option<LocalTime>),
    val: (FilterCallback<f64>, f64)
}

impl OutputFilter {
    fn builder() -> OutputFilterBuilder {
        OutputFilterBuilder {
            year:   (None, None),
            month:  (None, None),
            day:    (None, None),
            hour:   (None, None),
            val:    (None, 0.)
        }
    }

    async fn filter(&self, db: DbInternal) -> Vec<String> {
        let mut filtered_list = Vec::new();

        for set in db.iter() {
            let (key, val) = set.pair();
            
            if let Some(string) = self.filter_pair(key.clone(), *val).await {
                filtered_list.push(string);
            }
        }       

        filtered_list
    }

    async fn filter_pair(&self, key: DbKey, val: DbVal) -> Option<String> {
        let mut filters_failed = 0;
        let dt = DateTime::parse_from_str(&key, DATE_FORMAT).unwrap().naive_local();

        filters_failed += match &self.year.0 {
            Some(f) => if (*f)(dt, self.year.1.unwrap()) { 0 } else { 1 },
            None => 0
        };

        filters_failed += match &self.month.0 {
            Some(f) => if (*f)(dt, self.month.1.unwrap()) { 0 } else { 1 },
            None => 0
        };

        filters_failed += match &self.day.0 {
            Some(f) => if (*f)(dt, self.day.1.unwrap()) { 0 } else { 1 },
            None => 0
        };

        filters_failed += match &self.hour.0 {
            Some(f) => if (*f)(dt, self.hour.1.unwrap()) { 0 } else { 1 },
            None => 0
        };

        filters_failed += match &self.val.0 {
            Some(f) => if (*f)(val, self.val.1) { 0 } else { 1 },
            None => 0
        };
    
        if filters_failed == 0 {
            Some(format!("{}: {}\n", key, val))
        } else {
            None
        }
    }
}


impl OutputFilterBuilder {
    async fn add_filter(self, comp: Comp, filter_type: FilterType) -> OutputFilterBuilder {
        match filter_type {
            FilterType::Year(y) => self.year(comp, y).await,
            FilterType::Month(m) => self.month(comp, m).await,
            FilterType::Day(d) => self.day(comp, d).await,
            FilterType::Hour(h) => self.hour(comp, h).await,
            FilterType::Price(p) => self.val(comp, p).await
        }
    }

    async fn year(mut self, comp: Comp, yr: LocalTime) -> OutputFilterBuilder {
        self.year = (Self::construct_filter(comp).await, Some(yr));
        self
    }

    async fn month(mut self, comp: Comp, mnth: LocalTime) -> OutputFilterBuilder {
        self.month = (Self::construct_filter(comp).await, Some(mnth));
        self
    }

    async fn day(mut self, comp: Comp, day: LocalTime) -> OutputFilterBuilder {
        self.day = (Self::construct_filter(comp).await, Some(day));
        self
    }

    async fn hour(mut self, comp: Comp, hour: LocalTime) -> OutputFilterBuilder {
        self.hour = (Self::construct_filter(comp).await, Some(hour));
        self
    }

    async fn val(mut self, comp: Comp, val: f64) -> OutputFilterBuilder {
        self.val = (Self::construct_filter(comp).await, val);
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

    async fn construct_filter<T: PartialOrd>(comp: Comp) -> FilterCallback<T> {
        Some(match comp {
            Comp::Lesser        => lesser, 
            Comp::LesserEqual   => lesser_equal,
            Comp::Equal         => equal,
            Comp::GreaterEqual  => greater_equal, 
            Comp::Greater       => greater 
        })
    }
}

fn lesser<T: PartialOrd>(a: T, b: T) -> bool {
    return a < b;
}

fn lesser_equal<T: PartialOrd>(a: T, b: T) -> bool {
    return a <= b;
}

fn equal<T: PartialOrd>(a: T, b: T) -> bool {
    return a == b;
}

fn greater_equal<T: PartialOrd>(a: T, b: T) -> bool {
    return a >= b;
}

fn greater<T: PartialOrd>(a: T, b: T) -> bool {
    return a > b;
}

enum FilterType {
    Year(LocalTime),
    Month(LocalTime),
    Day(LocalTime),
    Hour(LocalTime),
    Price(f64)
}

impl FilterType {
    fn new_price(f_type: &str, val: f64) -> Option<Self> {
        match f_type.to_lowercase().as_str() {
            "price" => Some(Self::Price(val)),
            _       => None
        }   
    }

    fn new_time(f_type: &str, val: LocalTime) -> Option<Self> {
        match f_type.to_lowercase().as_str() {
            "year"  => Some(Self::Year(val)),
            "month" => Some(Self::Month(val)),
            "day"   => Some(Self::Day(val)),
            "hour"  => Some(Self::Hour(val)),
            _       => None 
        }
    }
}

enum Comp {
    Lesser,
    LesserEqual,
    Equal,
    GreaterEqual,
    Greater
}

impl Comp {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "<"     => Some(Comp::Lesser),
            "<="    => Some(Comp::LesserEqual),
            "=="    => Some(Comp::Equal),
            ">="    => Some(Comp::GreaterEqual),
            ">"     => Some(Comp::Greater),
            _       => None
        }
    }
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
