use std::collections::VecDeque;

use crate::problem::Problem;

use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
struct GridCell {
    val: i32,
    base: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportationSolver {
    problem: Problem,
    grid: Vec<Vec<GridCell>>,
    base: Vec<(usize, usize)>,
    pub stats: Option<SolverStats>,
    n: usize,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct SolverStats {
    objective: i32,
    iterations: usize,
    avg_chain_len: f32,
    n: usize,
}

impl TransportationSolver {
    pub fn new(problem: Problem) -> Self {
        let n = problem.supply.len();
        TransportationSolver {
            problem,
            grid: vec![vec![GridCell::default(); n]; n],
            base: vec![],
            stats: None,
            n,
        }
    }

    fn is_optimal(&self) -> bool {
        self.grid
            .iter()
            .flatten()
            .filter(|x| !x.base)
            .all(|x| x.val >= 0)
    }

    fn northwest(&mut self) {
        let (mut i, mut j) = (0, 0);
        let mut supply = self.problem.supply.clone();
        let mut demand = self.problem.demand.clone();
        loop {
            // Pick the maximum allowed goods
            let available = supply[i].min(demand[j]);

            // Update
            supply[i] -= available;
            demand[j] -= available;
            self.grid[i][j] = GridCell {val: available, base: true};
            self.base.push((i, j));

            // On bounds
            if i + 1 == self.n && j + 1 == self.n { break; }
            if i + 1 == self.n { j += 1; continue; }
            if j + 1 == self.n { i += 1; continue; }

            // Step
            if supply[i] == 0 { i += 1 }
            else if demand[j] == 0 { j += 1 }
            else { panic!("Either supply or demand value should be exhausted") }
        }

        debug_assert_eq!(supply.iter().sum::<i32>(), 0);
        debug_assert_eq!(demand.iter().sum::<i32>(), 0);
    }

    fn derive_steps(&self) -> (Vec<i32>, Vec<i32>) {
        let (mut u_assignments, mut v_assignments) = (vec![0; self.n], vec![0; self.n]);
        for (i, j) in self.base.iter() {
            u_assignments[*i] += 1;
            v_assignments[*j] += 1;
        }
        let (mut i_max, mut j_max) = (0, 0);
        let (mut i_max_val, mut j_max_val) = (u_assignments[0], v_assignments[0]);
        for i in 1..self.n {
            if i_max_val < u_assignments[i] {
                i_max_val = u_assignments[i];
                i_max = i
            }
            if j_max_val < v_assignments[i] {
                j_max_val = v_assignments[i];
                j_max = i
            }
        }

        let mut queue = VecDeque::<(usize, usize, bool)>::new();
        if i_max_val > j_max_val {
            for j in (0..self.n).filter(|x| self.grid[i_max][*x].base) {
                queue.push_back((i_max, j, false));
            }
        } else {
            for i in (0..self.n).filter(|x| self.grid[*x][j_max].base) {
                queue.push_back((i, j_max, true));
            }
        }

        let (mut u, mut v) = (vec![0; self.n], vec![0; self.n]);
        while !queue.is_empty() {
            let (i, j, set_u) = queue.pop_front().unwrap();
            if set_u {
                u[i] = self.problem.costs[i][j] - v[j];
                for j in (0..self.n).filter(|x| self.grid[i][*x].base && *x != j) {
                    queue.push_back((i, j, false));
                }
            } else {
                v[j] = self.problem.costs[i][j] - u[i];
                for i in (0..self.n).filter(|x| self.grid[*x][j].base && *x != i) {
                    queue.push_back((i, j, true));
                }
            }
        }
        (u, v)
    }

    fn fill_non_basic(&mut self, u: &[i32], v: &[i32]) {
        for i in 0..(u.len()) {
            for j in 0..(v.len()) {
                if !self.grid[i][j].base {
                    self.grid[i][j].val = self.problem.costs[i][j] - u[i] - v[j];
                }
            }
        }
    }

    /// Iterate over column j, without including row i
    fn col(&self, _i: usize, j: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..self.n).map(move |x| (x, j))
    }

    /// Iterate over row i, without including column j
    fn row(&self, i: usize, _j: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..self.n).map(move |x| (i, x))
    }

    /// Returns chain in reverse order, since it is more efficient to push to the end
    fn find_chain_rec(
        &self,
        visited: &mut Vec<Vec<bool>>,
        initial: (usize, usize),
        i: usize,
        j: usize,
        move_in_column: bool,
    ) -> Option<Vec<(usize, usize)>> {
        if visited[i][j] {
            return match initial == (i, j) {
                true => Some(vec![]),
                false => None
            }
        }
        visited[i][j] = true;

        // Can't aggregate row and column since they have different opaque types due to
        // generics. It could've been boxed, but it would affect efficiency.
        if move_in_column {
            for (next_i, next_j) in self.col(i, j).filter(|x| *x != (i, j) && (*x == initial || self.grid[x.0][x.1].base)) {
                if let Some(mut next) = self.find_chain_rec(visited, initial, next_i, next_j, !move_in_column) {
                    next.push((i, j));
                    return Some(next);
                }
            }
            None
        }
        else {
            for (next_i, next_j) in self.row(i, j).filter(|x| *x != (i, j) && (*x == initial || self.grid[x.0][x.1].base)) {
                if let Some(mut next) = self.find_chain_rec(visited, initial, next_i, next_j, !move_in_column) {
                    next.push((i, j));
                    return Some(next);
                }
            }
            None
        }
    }

    fn find_chain(&self) -> Vec<(usize, usize)>{
        // Find minimum coefficient
        let mut min_index = (0, 0);
        let mut min_value = self.grid[0][0].val;
        for i in 0..self.n {
            for j in 0..self.n {
                if !self.grid[i][j].base && self.grid[i][j].val < min_value {
                    min_value = self.grid[i][j].val;
                    min_index = (i, j);
                }
            }
        }

        let mut visited = vec![vec![false; self.n]; self.n];
        // It doesn't matter which direction to pick, since we always
        // arrive to the initial node
        let mut chain = self.find_chain_rec(&mut visited, min_index, min_index.0, min_index.1, true)
            .unwrap_or_else(|| panic!("Chain should always exist {visited:?}"));
        chain.reverse();
        chain
    }

    fn apply_chain(&mut self, chain: &[(usize, usize)]) {
        // First variable is not basic
        let (i, j) = chain[0];
        debug_assert!(!self.grid[i][j].base);

        // All other variables are basic
        debug_assert!(
            chain
            .iter()
            .map(|(i, j)| self.grid[*i][*j].base)
            .fold(true, |acc, x| acc & x)
        );

        // Find the variable that is leaving the base
        let mut min_index = 1;
        let mut min_value = self.grid[chain[1].0][chain[1].1].val;

        // skip(1) => Skip first variable
        // step_by(2) => Iterate only by donors
        for (idx, (i, j)) in chain.iter().enumerate().skip(1).step_by(2) {
            if self.grid[*i][*j].val < min_value {
                min_value = self.grid[*i][*j].val;
                min_index = idx;
            }
        }

        // Update donors
        for (i, j) in chain.iter().skip(1).step_by(2) {
            self.grid[*i][*j].val -= min_value;
        }

        // Update recipients
        for (i, j) in chain.iter().skip(2).step_by(2) {
            self.grid[*i][*j].val += min_value;
        }

        // Assign value for the variable entering the base
        let (i, j) = chain[0];
        self.grid[i][j].val = min_value;
        self.grid[i][j].base = true;

        // Remove from the base
        let (i, j) = chain[min_index];
        let base_pos = self.base.iter().position(|x| *x == chain[min_index]).expect("Inconsistency in bases");
        self.base[base_pos] = (chain[0].0, chain[0].1);
        self.grid[i][j].base = false;
    }

    fn objective(&self) -> i32 { 
        self.grid
            .iter()
            .flatten()
            .zip(self.problem.costs.iter().flatten())
            .filter(|(cell, _cost)| cell.base)
            .map(|(cell, cost)| cell.val * cost)
            .sum::<i32>()
    }

    pub fn solve(&mut self) {
        let mut iterations = 0;
        let mut chain_lengths = 0;
        self.northwest();

        loop {
            iterations += 1;

            let (u, v) = self.derive_steps();
            self.fill_non_basic(&u, &v);

            if self.is_optimal() { break }

            let chain = self.find_chain();
            chain_lengths += chain.len();
            self.apply_chain(&chain);
        }

        self.stats = Some(SolverStats {
            iterations,
            objective: self.objective(),
            avg_chain_len: chain_lengths as f32 / iterations as f32, 
            n: self.n
        });
    }
}
