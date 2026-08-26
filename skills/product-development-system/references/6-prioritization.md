# 6. Feature Prioritization

Support several methods. Every feature entering `Planned` carries at least one score in its
`priority:` block, and the method is named (never "by feel").

## RICE
```
        Reach × Impact × Confidence
RICE = ─────────────────────────────
                 Effort
```

## ICE
```
ICE = Impact × Confidence × Ease
```

## MoSCoW
Must have · Should have · Could have · Won't have.

## Value vs Effort
```
High Value / Low Effort   -> Do now
High Value / High Effort  -> Plan
Low Value  / Low Effort   -> Opportunistic
Low Value  / High Effort  -> Avoid
```

## Additional scores (optional inputs)
strategic alignment · user demand · revenue potential · retention impact ·
differentiation · urgency · technical debt reduction · risk reduction.

## Weighted score (the OmegaOS default when inputs exist)
```
Priority Score =
    User Value          × 25%
  + Business Value       × 20%
  + Strategic Alignment  × 20%
  + Confidence           × 15%
  + Urgency              × 10%
  + Feasibility          × 10%
```
`Strategic Alignment` is scored against the Vision Board pillars (ref 1). `Confidence` is the
overall discovery confidence (ref 5).

## How the agent uses it
- Pick the method that fits the decision: RICE/weighted for a ranked backlog, MoSCoW for a release
  cut, Value-vs-Effort for a quick triage. State which you used and show the numbers (R-CITE).
- A feature with no score does NOT enter `Planned`. "We should just do it" is not a priority.
- When ranking several features, compute the same method for all of them so the comparison is fair.
