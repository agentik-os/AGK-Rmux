from __future__ import annotations
from dataclasses import dataclass
from typing import Iterable

@dataclass
class Forecast:
    probability: float
    outcome: int
    domain: str = "general"

def brier_score(items: Iterable[Forecast]) -> float:
    xs = list(items)
    if not xs:
        raise ValueError("No resolved forecasts")
    return sum((x.probability - x.outcome) ** 2 for x in xs) / len(xs)

def calibration_buckets(items: Iterable[Forecast], width: float = .1):
    buckets = {}
    for x in items:
        lo = min(int(x.probability / width) * width, 1.0 - width)
        key = round(lo, 2)
        buckets.setdefault(key, []).append(x)
    return {k: {"n": len(v), "mean_p": sum(x.probability for x in v)/len(v), "frequency": sum(x.outcome for x in v)/len(v)} for k,v in sorted(buckets.items())}
