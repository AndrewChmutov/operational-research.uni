import json
import sys
from dataclasses import dataclass

from pulp import PULP_CBC_CMD, LpConstraint, LpConstraintEQ, LpProblem, LpVariable, lpSum

EPSILON = 10**(-6)
M = 5 * 10**6


# Define models
@dataclass
class GridCell:
    val: int
    base: bool


@dataclass
class Problem:
    costs: list[list[int]]
    supply: list[int]
    demand: list[int]


@dataclass
class SolverStats:
    objective: int
    iterations: int
    avg_chain_len: float
    n: int


@dataclass
class Solver:
    problem: Problem
    grid: list[list[GridCell]]
    base: list[tuple[int, int]]
    stats: SolverStats
    n: int

    def __init__(self, **kwargs) -> None:
        self.problem = Problem(**kwargs["problem"])
        self.grid = [[GridCell(**cell) for cell in row] for row in kwargs["grid"]]
        self.base = kwargs["base"]
        self.stats = SolverStats(**kwargs["stats"])
        self.n = kwargs["n"]


# Load models
solver = Solver(**json.load(sys.stdin))
try:
    pass
except Exception:
    print("Couldn't parse the json")
    sys.exit(1)


# Create LP problem
model = LpProblem(name="Transportation_problem")
n = len(solver.problem.supply)

variables = [
    [
        LpVariable(f"X_{i}_{j}", lowBound=0)
        for j in range(n)
    ]
    for i in range(n)
]

# Supply constraints
for i in range(n):
    model += LpConstraint(
        e=lpSum([
            variables[i][j]
            for j in range(n)
        ]),
        sense=0,
        name=f"Supply_constraint {i}",
        rhs=solver.problem.supply[i],
    )

# Demand constraints
for j in range(n):
    model += LpConstraint(
        e=lpSum([
            variables[i][j]
            for i in range(n)
        ]),
        sense=LpConstraintEQ,
        name=f"Demand_constraint_{j} ",
        rhs=solver.problem.demand[j],
    )

# Objective
model += lpSum([
    variables[i][j] * solver.problem.costs[i][j]
    for i in range(n) for j in range(n)
])

# Solve
model.solve(PULP_CBC_CMD(msg=False))
optimum: float = model.objective.value()  # pyright: ignore

# Debug information
print()
print(f"Found: {solver.stats.objective}")
print(f"Correct: {optimum} ", end="")

# Fail if not optimum
is_close = abs(optimum - solver.stats.objective) > EPSILON
both_inf = optimum > M and solver.stats.objective > M
if is_close and not both_inf:
    print("Custom solver did not reach the optimal solution")
    sys.exit(1)
