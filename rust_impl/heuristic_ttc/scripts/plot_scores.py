import argparse
from pathlib import Path

import matplotlib.pyplot as plt


def main() -> None:
    parser = argparse.ArgumentParser(description="Plot local search score progression.")
    parser.add_argument(
        "--input",
        default="local_search_scores.csv",
        help="Path to the CSV written by local_search",
    )
    parser.add_argument(
        "--output",
        default="local_search_scores.png",
        help="Output image path",
    )
    args = parser.parse_args()

    input_path = Path(args.input)
    if not input_path.exists():
        raise SystemExit(f"Missing input file: {input_path}")

    iterations = []
    scores = []
    with input_path.open("r", encoding="utf-8") as handle:
        header = handle.readline()
        if not header:
            raise SystemExit("Empty input file")
        for line in handle:
            line = line.strip()
            if not line:
                continue
            iter_str, score_str = line.split(",", 1)
            iterations.append(int(iter_str))
            scores.append(int(score_str))

    plt.figure(figsize=(10, 5))
    plt.plot(iterations, scores, linewidth=1.5)
    plt.title("Local Search Best Score Over Time")
    plt.xlabel("Iteration")
    plt.ylabel("Best Score")
    plt.tight_layout()
    plt.savefig(args.output, dpi=150)
    print(f"Wrote plot to {args.output}")


if __name__ == "__main__":
    main()
