import json
import sys
from dataclasses import dataclass

import matplotlib.pyplot as plt
import seaborn

seaborn.set_theme()

EPSILON = 10 ** (-6)


@dataclass
class SolverStats:
    objective: int
    iterations: int
    avg_chain_len: float
    n: int


try:
    stats: list[list[SolverStats]] = [
        [SolverStats(**s) for s in bunch] for bunch in json.load(sys.stdin)
    ]
except Exception:
    print("Couldn't parse the json")
    sys.exit(1)

sizes = [s[0].n for s in stats]
mean_iterations = [sum(s.iterations for s in stat) / len(stat) for stat in stats]
mean_chain_lengths = [sum(s.avg_chain_len for s in stat) / len(stat) for stat in stats]

fig, ax = plt.subplots(1, 2)
plt.suptitle("Stonks")
ax[0].plot(sizes, mean_iterations)
ax[0].set_title("Iterations")
ax[1].plot(sizes, mean_chain_lengths)
ax[1].set_title("Average chain length")
plt.savefig("target/stats.png")
