use descriptor::SudiceExpression;
use interpreter;

use rand;

use std::vec::Vec;
use std::fmt;

const OBS_FACTOR: usize = 2000;

#[derive(Debug)]
pub struct SudiceResults {
    pub total: usize,
    pub min: i64,
    pub max: i64,
    pub hist: Vec<u64>,
    pub dist: Vec<f64>,
    pub ev: f64,
    pub sd: f64,
}

impl fmt::Display for SudiceResults {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "##### [ RESULTS ] #####")?;
        writeln!(f, "Minimum:\t{}", self.min)?;
        writeln!(f, "Maximum:\t{}", self.max)?;
        writeln!(f, "Range:\t\t{}", self.max - self.min)?;
        writeln!(f, "Observations:\t{}", self.total)?;
        writeln!(f, "Expected Value:\t{}", self.ev)?;
        writeln!(f, "Std. Deviation:\t{}", self.sd)?;
        writeln!(f, "##### [ DISTRIBUTION ] #####")?;
        let ichars = 1 + (self.max as f64).log10().ceil() as usize;
        let fchars = (self.total as f64).log10().ceil() as usize; 
        let mhist = *(self.hist.iter().max().unwrap()) as f64;
        for i in self.min..self.max+1 {
            let idx = (i - self.min) as usize;
            let dprop = (self.hist[idx] as f64) / mhist;
            write!(f, "{n:>width$} |", n = i, width = ichars)?;
            write!(f, "{freq:>width$} ", freq = self.hist[idx] as usize, width = fchars)?;
            writeln!(f, "| {prop:>width$}%", prop = (self.dist[idx] * 100.0).round(), width = (20.0 * dprop).round() as usize)?;
        }
        Ok(())
    }
}

pub fn estimate(code: &SudiceExpression, min: i64, max: i64) -> SudiceResults {
    let size = (max - min + 1) as usize;
    let mut hist: Vec<u64> = Vec::with_capacity(size);
    hist.resize(size, 0);
    let mut rng = rand::thread_rng();
    let total = size * OBS_FACTOR;
    for _ in 0..total {
        let s = interpreter::interpret(&code, &mut rng);
        hist[(s - min) as usize] += 1;
    }

    // Compute exp. value and cache distribution
    let mut dist: Vec<f64> = Vec::with_capacity(size);
    let mut ev = 0.0 as f64;
    for i in min..max+1 {
        let div = (hist[(i - min) as usize] as f64) / (total as f64);
        dist.push(div);
        ev += i as f64 * div;
    }

    // Compute std. dev
    let mut sd = 0.0 as f64;
    for i in min..max+1 {
        sd += (i as f64 - ev) * (i as f64 - ev) * dist[(i - min) as usize];
    }
    sd = sd.sqrt();
    SudiceResults { total: total, min: min, max: max, hist: hist, dist: dist, ev: ev, sd: sd }
}
