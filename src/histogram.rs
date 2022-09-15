use std::collections::BTreeMap;

pub struct Histogram<'a, T> {
    title: &'a str,
    data_set: BTreeMap<T, i32>,
    total: u32,
}

impl<'a, T: std::cmp::Ord + std::fmt::Debug> Histogram<'a, T> {
    pub fn new(title: &'a str) -> Self {
        Histogram {
            title,
            data_set: BTreeMap::new(),
            total: 0,
        }
    }

    pub fn with_domain<U>(title: &'a str, domain: U) -> Self
    where
        U: IntoIterator<Item = T>,
    {
        let mut set: BTreeMap<T, i32> = BTreeMap::new();
        for item in domain {
            set.insert(item, 0);
        }

        Histogram {
            title,
            data_set: set,
            total: 0,
        }
    }

    pub fn inc(&mut self, item: T) {
        *self.data_set.entry(item).or_insert(0) += 1;
        self.total += 1;
    }

    #[allow(dead_code)]
    pub fn dec(&mut self, item: &T) {
        if let Some(count) = self.data_set.get_mut(item) {
            *count -= 1;
            self.total -= 1;
        }
    }

    pub fn show(&self, scale: usize) {
        let scale = if scale > 0 { scale } else { 1 };

        println!("{}", self.title);
        println!("{:=<1$}", "", 60);
        for (item, count) in &self.data_set {
            // Prevent integer division from truncating to zero if there is at least
            // one instance of an item
            let scaled: usize = if (*count as usize) < scale && *count > 0 {
                1
            } else {
                (*count as usize) / scale
            };

            println!("{: >5?}|{:*<2$} ({3})", item, "", scaled, *count as usize);
        }
        println!();
    }

    pub fn show_percent(&self, scale: usize) {
        let scale = if scale > 0 { scale } else { 1 };

        println!("{}", self.title);
        println!("{:=<1$}", "", 60);
        for (item, count) in &self.data_set {
            // Prevent integer division from truncating to zero if there is at least
            // one instance of an item
            let scaled: usize = if (*count as usize) < scale && *count > 0 {
                1
            } else {
                (*count as usize) / scale
            };

            let percent = (*count as f64 / self.total as f64) * 100.0;

            println!("{: >5?}|{:*<2$} ({3:.2}%)", item, "", scaled, percent);
            // println!("{: >5?}|{:*<2$} ({3}/{4}={5:.2}%)", item, "", scaled, *count as usize, self.total, percent);
        }
        println!();
    }
}
