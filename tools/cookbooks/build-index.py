#!/usr/bin/env python3
"""Build the OmegaOS cookbook index from an anthropics/claude-cookbooks checkout.

The index is the DISCOVERY half of the integration and ships in the OmegaOS
repo (~40KB): every fresh clone can find the right Anthropic reference recipe
even when the 70M notebook corpus was never cloned. `install-cookbooks.sh`
installs the CORPUS and is optional; this index is not.

Each recipe carries the upstream URL pinned to the recorded commit, so a row is
actionable with or without a local corpus.

    build-index.py <path-to-cookbooks-checkout> [-o recipes.json]
"""
import argparse
import datetime
import hashlib
import json
import os
import subprocess
import sys

UPSTREAM = "https://github.com/anthropics/claude-cookbooks"

# registry.yaml categories -> the need an OmegaOS agent actually types. The RAG
# embeds this text, so the phrasing is load-bearing: it is what makes
# "how do I evaluate my prompt" retrieve the Evals recipes.
CATEGORY_INTENT = {
    "RAG & Retrieval": "retrieval augmented generation, RAG pipeline, vector search, embeddings, reranking, chunking, contextual retrieval, knowledge base grounding",
    "Agent Patterns": "agent workflow architecture, prompt chaining, routing, parallelization, orchestrator workers, evaluator optimizer, multi-agent orchestration, subagents",
    "Evals": "evaluation harness, benchmark, LLM judge, grading, test set, measuring prompt quality, regression detection",
    "Tools": "tool use, function calling, tool definitions, structured output, JSON mode, tool result handling",
    "Multimodal": "vision, images, charts, diagrams, PDF parsing, document extraction, transcription, OCR",
    "Claude Managed Agents": "server-hosted managed agents, sandboxed environments, sessions, file mounts, webhooks, budget caps, human in the loop gating",
    "Claude Agent SDK": "Claude Agent SDK, building a custom coding agent, hosting an agent, docker, kubernetes, production deployment",
    "Integrations": "third party integration, Pinecone, MongoDB, Voyage embeddings, Wikipedia, Slack, database connection",
    "Responses": "response shaping, output formatting, streaming, structured responses, citations, refusal handling",
    "Observability": "tracing, logging, monitoring an agent, telemetry, debugging agent runs",
    "Cybersecurity": "security analysis, vulnerability detection, threat modelling, code security review",
    "Thinking": "extended thinking, reasoning budgets, chain of thought, deliberation",
    "Fine-Tuning": "fine-tuning, model customisation, training data preparation",
    "Skills": "Claude Skills, SKILL.md authoring, progressive disclosure, bundled skill files",
    "Prompting": "prompt engineering, prompt caching, system prompts, few-shot examples",
    "Cost": "cost optimisation, token budget, latency, model selection, caching for cost",
}


def git(repo, *args):
    return subprocess.run(
        ["git", "-C", repo, *args], capture_output=True, text=True, check=True
    ).stdout.strip()


def load_registry(repo):
    path = os.path.join(repo, "registry.yaml")
    if not os.path.isfile(path):
        sys.exit(f"error: no registry.yaml in {repo} (is this a claude-cookbooks checkout?)")
    try:
        import yaml
    except ImportError:
        sys.exit("error: PyYAML required — pip install pyyaml")
    with open(path, encoding="utf-8") as handle:
        data = yaml.safe_load(handle)
    if not isinstance(data, list) or not data:
        sys.exit("error: registry.yaml did not parse to a non-empty list")
    return data


def slug(path):
    base = os.path.basename(path)
    for ext in (".ipynb", ".md", ".py"):
        if base.endswith(ext):
            base = base[: -len(ext)]
    return base.replace("_", "-").replace(" ", "-").lower()


def build(repo, out):
    entries = load_registry(repo)
    sha = git(repo, "rev-parse", "HEAD")
    recipes = []
    seen = set()
    for entry in entries:
        path = entry.get("path")
        title = entry.get("title")
        desc = (entry.get("description") or "").strip().replace("\n", " ")
        if not path or not title:
            continue
        name = slug(path)
        # registry paths are unique, but slugs can collide across directories
        if name in seen:
            name = f"{slug(os.path.dirname(path))}-{name}"
        seen.add(name)
        cats = entry.get("categories") or []
        intent = " ".join(CATEGORY_INTENT.get(c, c.lower()) for c in cats)
        recipes.append(
            {
                "name": name,
                "title": title,
                "description": desc,
                "path": path,
                "categories": cats,
                "group": cats[0] if cats else "Cookbook",
                "intent": intent,
                "url": f"{UPSTREAM}/blob/{sha}/{path}",
                "exists": os.path.isfile(os.path.join(repo, path)),
            }
        )
    recipes.sort(key=lambda r: (r["group"], r["name"]))
    missing = [r["name"] for r in recipes if not r["exists"]]
    payload = {
        "schema_version": 1,
        "generated": datetime.date.today().isoformat(),
        "upstream": UPSTREAM,
        "commit": sha,
        "commit_date": git(repo, "log", "-1", "--format=%cI"),
        "license": "MIT (c) Anthropic",
        "count": len(recipes),
        "recipes": recipes,
    }
    payload["index_hash"] = hashlib.sha256(
        json.dumps(payload["recipes"], sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()
    tmp = out + ".tmp"
    with open(tmp, "w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, ensure_ascii=False, sort_keys=True)
        handle.write("\n")
    os.replace(tmp, out)
    print(f"cookbook index: {len(recipes)} recipes @ {sha[:7]} -> {out}")
    if missing:
        print(f"warning: {len(missing)} registry paths absent from checkout: {missing[:5]}",
              file=sys.stderr)
    return payload


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("repo", help="path to an anthropics/claude-cookbooks checkout")
    ap.add_argument("-o", "--out",
                    default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "recipes.json"))
    args = ap.parse_args()
    build(os.path.abspath(args.repo), os.path.abspath(args.out))


if __name__ == "__main__":
    main()
