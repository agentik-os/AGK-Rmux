"""The execution DAG. Hard dependencies are authoritative edges; the graph
refuses cycles at construction (pack rule: dependencies are authoritative,
a cyclic plan is not a plan)."""

from __future__ import annotations

import networkx as nx

from .models import StepSpec


class StepGraph:
    def __init__(self, steps: list[StepSpec]):
        self.steps = {s.step_id: s for s in steps}
        self.graph = nx.DiGraph()
        for step in steps:
            self.graph.add_node(step.step_id)
        for step in steps:
            for dep in step.dependencies.hard:
                if dep not in self.steps:
                    raise ValueError(f"Missing dependency {dep} for {step.step_id}")
                self.graph.add_edge(dep, step.step_id)
        if not nx.is_directed_acyclic_graph(self.graph):
            cycle = nx.find_cycle(self.graph)
            pretty = " -> ".join(edge[0] for edge in cycle) + f" -> {cycle[0][0]}"
            raise ValueError(f"Step graph contains a cycle: {pretty}")

    def hard_dependencies(self, step_id: str) -> list[str]:
        return list(self.graph.predecessors(step_id))

    def downstream(self, step_id: str) -> set[str]:
        return nx.descendants(self.graph, step_id)

    def downstream_weight(self, step_id: str) -> float:
        """Total weight this step unlocks (its own weight excluded)."""
        return sum(self.steps[s].weight for s in self.downstream(step_id))

    def critical_path_weight(self, step_id: str) -> float:
        """Weight of the heaviest dependency chain STARTING at step_id
        (inclusive). The planner uses it to prefer steps sitting on the
        project's longest remaining chain."""
        memo: dict[str, float] = {}

        def walk(node: str) -> float:
            if node in memo:
                return memo[node]
            succ = list(self.graph.successors(node))
            best = max((walk(s) for s in succ), default=0.0)
            memo[node] = self.steps[node].weight + best
            return memo[node]

        return walk(step_id)

    def critical_path(self) -> list[str]:
        """The heaviest weighted chain across the whole graph."""
        if not self.steps:
            return []
        best_start = max(self.steps, key=self.critical_path_weight)
        path = [best_start]
        node = best_start
        while True:
            succ = list(self.graph.successors(node))
            if not succ:
                return path
            node = max(succ, key=self.critical_path_weight)
            path.append(node)
