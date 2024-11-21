use rand::prelude::*;
use rand::distributions::Uniform;
use serde::{Serialize, Deserialize};

pub const M: i32 = 5_000_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Problem {
    pub costs: Vec<Vec<i32>>,
    pub supply: Vec<i32>,
    pub demand: Vec<i32>,
}

pub struct GenConfig {
    pub max_value: i32,
    pub m_val_probability: f32,
    pub zero_val_probability: f32,
    pub zero_col_row_max_fraction: f32,
}

impl Default for GenConfig {
    fn default() -> Self {
        Self {
            max_value: 10,
            m_val_probability: 0.1,
            zero_val_probability: 0.02,
            zero_col_row_max_fraction: 0.2,
        }
    }
}

impl GenConfig {
    pub fn gen(&self, n: usize) -> Problem {
        let mut rng = thread_rng();

        // Supply
        let dist = Uniform::new_inclusive(1, self.max_value);
        let mut supply = Vec::with_capacity(n);
        let mut total_supply = 0;
        let mut temp;
        for _ in 0..n {
            temp = dist.sample(&mut rng);
            supply.push(temp);
            total_supply += temp;
        }

        // Demand
        let mut demand_f = Vec::with_capacity(n);
        let mut total_demand_f = 0.0;
        let mut temp;
        for _ in 0..n {
            temp = rng.gen::<f32>();
            demand_f.push(temp);
            total_demand_f += temp;
        }
        let mut demand = Vec::with_capacity(n);
        let mut total_demand = 0;
        let mut temp;
        for d in demand_f[..(n - 1)].iter() {
            temp = (d / total_demand_f * total_supply as f32) as i32;
            demand.push(temp);
            total_demand += temp;
        }
        demand.push(total_supply - total_demand);

        // Costs
        let mut costs = vec![vec![0; n]; n];
        let rows = n - (rng.gen_range(0.0..self.zero_col_row_max_fraction) * (n as f32)) as usize;
        let cols = n - (rng.gen_range(0.0..self.zero_col_row_max_fraction) * (n as f32)) as usize;
        for i in 0..rows {
            for j in 0..cols {
                costs[i][j] = rng.gen_range(1..=self.max_value);
            }
        }

        let zero_val_int = 0.0..self.zero_val_probability;
        let m_val_int = self.zero_val_probability..(self.zero_val_probability + self.m_val_probability);
        for i in 0..n {
            for j in 0..n {
                let roll = rng.gen();
                if zero_val_int.contains(&roll) {
                    costs[i][j] = 0
                } else if m_val_int.contains(&roll) {
                    costs[i][j] = M
                }
            }
        }

        // Verify
        debug_assert_eq!(supply.iter().sum::<i32>(), demand.iter().sum::<i32>(), "Not feasible");
        debug_assert!(supply.iter().fold(true, |acc, x| acc & (x >= &0)), "Non-negativity constrant is violated for supply");
        debug_assert!(demand.iter().fold(true, |acc, x| acc & (x >= &0)), "Non-negativity constrant is violated for demand");

        Problem { costs, supply, demand }
    }
}
