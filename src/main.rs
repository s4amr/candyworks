use std::{
    collections::{HashSet, VecDeque},
    fmt,
    time::Instant,
};

use rustyline::DefaultEditor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Candies {
    eggs: i32,
    worms: i32,
    cakes: i32,
    fishes: i32,
    meats: i32,
}

impl Candies {
    pub fn trade(&self, trade: &Trade) -> Option<Candies> {
        let result = Candies {
            eggs: self.eggs + trade.receive.eggs - trade.give.eggs,
            meats: self.meats + trade.receive.meats - trade.give.meats,
            fishes: self.fishes + trade.receive.fishes - trade.give.fishes,
            worms: self.worms + trade.receive.worms - trade.give.worms,
            cakes: self.cakes + trade.receive.cakes - trade.give.cakes,
        };
        if result.eggs < 0
            || result.meats < 0
            || result.fishes < 0
            || result.worms < 0
            || result.cakes < 0
        {
            None
        } else {
            Some(result)
        }
    }

    pub fn total(&self) -> i32 {
        self.eggs + self.worms + self.cakes + self.fishes + self.meats
    }

    pub fn contains(&self, other: &Candies) -> bool {
        self.eggs >= other.eggs
            && self.meats >= other.meats
            && self.fishes >= other.fishes
            && self.worms >= other.worms
            && self.cakes >= other.cakes
    }

    pub fn none() -> Candies {
        Candies {
            eggs: 0,
            worms: 0,
            cakes: 0,
            fishes: 0,
            meats: 0,
        }
    }

    pub fn add_by_index(&mut self, index: usize, value: i32) {
        match index {
            0 => self.eggs += value,
            1 => self.worms += value,
            2 => self.cakes += value,
            3 => self.fishes += value,
            4 => self.meats += value,
            _ => (),
        }
    }

    fn display(&self, include_zeros: bool) -> String {
        let mut result = Vec::new();
        result.push((self.eggs, "egg", "eggs"));
        result.push((self.worms, "worm", "worms"));
        result.push((self.cakes, "cake", "cakes"));
        result.push((self.fishes, "fish", "fishes"));
        result.push((self.meats, "meat", "meats"));
        if !include_zeros {
            result.retain(|(count, _, _)| *count != 0);
        }
        result
            .into_iter()
            .map(|(count, s, p)| format!("{:2} {}", count, if count == 1 { s } else { p }))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl fmt::Display for Candies {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display(false))
    }
}

#[derive(Debug, Clone, Copy)]
struct Trade {
    give: Candies,
    receive: Candies,
}

impl Trade {
    pub fn standard_trade(a: usize, b: usize) -> Self {
        let mut give = Candies::none();
        let mut receive = Candies::none();
        give.add_by_index(a, 3);
        receive.add_by_index(b, 1);
        Trade { give, receive }
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.give, self.receive)
    }
}

struct CandyWorks {
    candies: Candies,
    max_candies: u32,
    trades: Vec<Trade>,
    combinations: Vec<(Candies, Option<(usize, Trade)>)>,
}

impl CandyWorks {
    pub fn new(candies: Candies, max_candies: u32, custom_trades: Vec<Trade>) -> Self {
        let mut trades = custom_trades;
        for i in 0..5 {
            for j in 0..5 {
                if i != j {
                    trades.push(Trade::standard_trade(i, j));
                }
            }
        }
        CandyWorks {
            candies,
            max_candies,
            trades,
            combinations: Vec::new(),
        }
    }

    pub fn explore(&mut self) {
        let t = Instant::now();
        let mut collections = vec![(self.candies, None)];
        let mut known_sets = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_front(0);
        known_sets.insert(self.candies);
        while let Some(index) = queue.pop_back() {
            let (candies, _) = collections[index];
            let mut new_collections = Vec::new();
            for trade in &self.trades {
                if let Some(new_candies) = candies.trade(trade) {
                    let total = new_candies.total();
                    if total <= self.max_candies as i32 && !known_sets.contains(&new_candies) {
                        new_collections.push((new_candies, Some((index, *trade))));
                        known_sets.insert(new_candies);
                    }
                }
            }
            queue.extend((0..new_collections.len()).map(|i| i + collections.len()));
            collections.append(&mut new_collections);
        }
        self.combinations = collections;
        println!("Elapsed time: {:?}", t.elapsed());
    }

    pub fn stadistics(&self) {
        if self.combinations.is_empty() {
            println!("No combinations found");
            return;
        }
        println!("Total combinations: {}", self.combinations.len());
        let min_candies = self
            .combinations
            .iter()
            .map(|(candies, _)| candies.total())
            .min()
            .unwrap();
        let max_candies = self
            .combinations
            .iter()
            .map(|(candies, _)| candies.total())
            .max()
            .unwrap();
        println!("Min candies: {}", min_candies);
        println!("Max candies: {}", max_candies);
        let max_trades = self
            .combinations
            .iter()
            .filter(|(_, parent)| parent.is_some())
            .map(|(_, parent)| self.len_from_combination(parent.as_ref().unwrap().0))
            .max()
            .unwrap();
        println!("Max trades: {}", max_trades);
    }

    pub fn len_from_combination(&self, index: usize) -> usize {
        let mut current = index;
        let mut len = 0;
        while let Some((_, parent)) = self.combinations.get(current) {
            len += 1;
            if let Some((i, _)) = parent {
                current = *i;
            } else {
                break;
            }
        }
        len
    }

    pub fn find_optimal_route(&self, target: Candies) -> Option<Vec<Trade>> {
        if self.candies.contains(&target) {
            return Some(Vec::new());
        }
        let mut candidates = self
            .combinations
            .iter()
            .filter(|(candies, _)| candies.contains(&target))
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            return None;
        }
        let max = candidates
            .iter()
            .map(|(candies, _)| candies.total())
            .max()
            .unwrap();
        candidates.retain(|(candies, _)| candies.total() == max);
        let mut result = Vec::new();
        let mut current = candidates[0].1;
        while let Some((parent, trade)) = current {
            result.push(trade);
            current = self.combinations[parent].1;
        }
        result.reverse();
        Some(result)
    }
}

fn main() {
    let mut candies = Candies::none();
    let mut rl = DefaultEditor::new().unwrap();
    let names = ["eggs", "worms", "cakes", "fishes", "meats"];
    for (i, name) in names.iter().enumerate() {
        println!("How many {} do you have?", name);
        let input = rl.readline(">> ").unwrap();
        let value = input.trim().parse::<i32>().unwrap();
        candies.add_by_index(i, value);
    }
    let mut trades = Vec::new();
    println!("Use E for eggs, W for worms, C for cakes, F for fishes and M for meats");
    for _ in 0..3 {
        let input = rl.readline("Trade give: ").unwrap();
        let mut give = Candies::none();
        for c in input.to_lowercase().chars() {
            match c {
                'e' => give.eggs += 1,
                'w' => give.worms += 1,
                'c' => give.cakes += 1,
                'f' => give.fishes += 1,
                'm' => give.meats += 1,
                _ => (),
            }
        }
        let input = rl.readline("Trade receive: ").unwrap();
        let mut receive = Candies::none();
        for c in input.to_lowercase().chars() {
            match c {
                'e' => receive.eggs += 1,
                'w' => receive.worms += 1,
                'c' => receive.cakes += 1,
                'f' => receive.fishes += 1,
                'm' => receive.meats += 1,
                _ => (),
            }
        }
        trades.push(Trade { give, receive });
    }
    let mut candy_works = CandyWorks::new(candies, 20, trades);
    candy_works.explore();
    candy_works.stadistics();

    let mut target = Candies::none();
    for (i, name) in names.iter().enumerate() {
        println!("How many {} do you want?", name);
        let input = rl.readline(">> ").unwrap();
        let value = input.trim().parse::<i32>().unwrap();
        target.add_by_index(i, value);
    }
    let route = candy_works.find_optimal_route(target);
    if let Some(route) = route {
        let mut previous = candy_works.candies;
        for trade in route {
            println!("({}) {}", previous.display(true), trade);
            previous = previous.trade(&trade).unwrap();
        }
        println!("({})", previous.display(true));
    } else {
        println!("No route found");
    }
}
