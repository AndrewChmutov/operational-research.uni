use std::io::Write;
use std::process::{Command, Stdio};

use crate::problem::*;
use crate::solver::*;

fn verify(solver: &TransportationSolver) {
    // Spawn the command
    let mut child = Command::new(".venv/bin/python3")
        .stdin(Stdio::piped())
        .args(["scripts/check.py"])
        .spawn()
        .expect("Failed to start a subprocess");

    // Pass the data
    child
        .stdin
        .as_mut()
        .expect("Failed to pass data to the checker")
        .write_all(
            serde_json::to_string(&solver)
                .expect("Failed to serialize a solver")
                .as_bytes(),
        )
        .expect("Failed to write to subprocess stdin");

    // Check the status code
    match child
        .wait_with_output()
        .expect("Failed to read stdout")
        .status
        .code()
        .expect("Couldn't retrieve")
    {
        0 => (),
        _ => panic!("Wrong answer"),
    }
}

fn solve_problem(problem: Problem, check: bool) -> TransportationSolver {
    let mut solver = TransportationSolver::new(problem);
    solver.solve();
    if check {
        verify(&solver);
    }
    solver
}

pub fn solve_exercise(check: bool) -> TransportationSolver {
    solve_problem(
        Problem {
            costs: vec![
                vec![7, 5, 5, 0],
                vec![3, 10, 10, M],
                vec![3, 10, 10, 0],
                vec![M, M, 0, 0],
            ],
            supply: vec![30, 20, 80, 80],
            demand: vec![40, 40, 20, 110],
        },
        check,
    )
}

pub fn solve_former_no_path(check: bool) -> TransportationSolver {
    solve_problem(
        Problem {
            costs: vec![
                vec![7, 10, M, 10],
                vec![5, 4, 7, 4],
                vec![4, 6, 8, 4],
                vec![M, 1, 4, 4],
            ],
            supply: vec![1, 6, 10, 8],
            demand: vec![M, 1, 4, 13],
        },
        check,
    )
}

pub fn solve_former_not_optimal(check: bool) -> TransportationSolver {
    solve_problem(
        Problem {
            costs: vec![
                vec![7, 10, M, 10],
                vec![5, 4, 7, 4],
                vec![4, 6, 8, 4],
                vec![M, 1, 4, 4],
            ],
            supply: vec![1, 6, 10, 8],
            demand: vec![6, 2, 4, 13],
        },
        check,
    )
}

fn solve_generated(n: usize, check: bool) -> TransportationSolver {
    solve_problem(GenConfig::default().gen(n), check)
}

#[cfg(test)]
mod check {
    use super::*;
    use rstest::*;

    #[rstest]
    fn exercise() {
        solve_exercise(true);
    }
    #[rstest]
    fn former_no_path() {
        solve_former_no_path(true);
    }
    #[rstest]
    fn former_not_optimal() {
        solve_former_not_optimal(true);
    }

    #[rstest]
    #[case(6)]
    #[case(10)]
    #[case(50)]
    #[case(100)]
    #[case(200)]
    fn generated(#[case] n: usize) {
        solve_generated(n, true);
    }
}

#[cfg(test)]
mod perf {
    use super::*;
    use rstest::*;

    #[rstest]
    fn exercise() {
        solve_exercise(false);
    }
    #[rstest]
    fn former_no_path() {
        solve_former_no_path(false);
    }
    #[rstest]
    fn former_not_optimal() {
        solve_former_not_optimal(false);
    }

    #[rstest]
    #[case(6)]
    #[case(10)]
    #[case(50)]
    #[case(100)]
    #[case(200)]
    fn generated(#[case] n: usize) {
        solve_generated(n, false);
    }
}

#[cfg(test)]
mod artifact {
    use super::*;
    use rstest::*;

    #[rstest]
    fn create_artifact() {
        let stats = (5..100)
            .step_by(2)
            .map(|x| {
                (0..10)
                    .map(|_| solve_generated(x, false).stats.unwrap())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let mut child = Command::new(".venv/bin/python3")
            .stdin(Stdio::piped())
            .args(["scripts/plot.py"])
            .spawn()
            .expect("Failed to start a subprocess");

        child
            .stdin
            .as_mut()
            .expect("Failed to pass data to the plotting script")
            .write_all(
                serde_json::to_string(&stats)
                    .expect("Failed to serialize statistics")
                    .as_bytes(),
            )
            .expect("Failed to write to subprocess stdin");
    }
}
