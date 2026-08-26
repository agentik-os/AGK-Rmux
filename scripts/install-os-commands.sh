#!/usr/bin/env bash
# Install the canonical Claude and Codex root commands for all OmegaOS products.
# This script is intentionally standalone so install parity can exercise the
# generated files in an isolated HOME instead of merely grepping install.sh.

set -euo pipefail

os_command_catalog() {
    cat <<'CATALOG'
mindset-os|mindset-os|mindset,mindset-os
health-energy-os|health-energy-os|health,health-energy-os
habit-tracker-os|habit-tracker-os|habits,habits-os,habit-tracker-os
alignment-os|alignment-os|coach,align,alignment-os
strategy-portfolio-os|strategy-portfolio-os|strategy,strategy-portfolio-os
brainstorm-os|brainstorm-os|brainstorm,brainstorm-os
market-research-os|market-research-os|market-research,market-research-os
blueprint-os|blueprint-os|blueprint,blueprint-os
design-os|design-os|design-os
stepper-os|stepper-os|stepper,stepper-os
builder-os|builder-os|build,builder-os
quality-evaluation-release-os|quality-evaluation-release-os|quality,quality-evaluation-release-os
storyteller-os|storyteller-os|story,storyteller-os
revenue-os|revenue-os|revenue,revenue-os
delivery-customer-success-os|delivery-customer-success-os|delivery,delivery-customer-success-os
relationship-network-os|relationship-network-os|network,relationship-network-os
wealth-capital-os|wealth-capital-os|wealth,wealth-capital-os
execution-os|execution-os|execute,execution-os
operations-automation-os|operations-automation-os|operations,operations-automation-os
review-governance-os|review-governance-os|review,review-governance-os
context-memory-os|context-memory-os|memory,context-memory-os
ai-logic-os|ai-logic-os|ai-logic,ailogic,ai-logic-os
content-os|content-os|content,content-os
books-os|alexandria|books-os
CATALOG
}

if [[ "${1:-}" == "--list" ]]; then
    os_command_catalog
    exit 0
fi

omega_dir="${OMEGA_DIR:-$HOME/.omega}"
claude_dir="${CLAUDE_COMMAND_DIR:-$HOME/.claude/commands}"
codex_dir="${CODEX_PROMPT_DIR:-$HOME/.codex/prompts}"
mkdir -p "$claude_dir" "$codex_dir"

installed=0
while IFS='|' read -r os_slug skill_slug aliases; do
    skill_file="$omega_dir/skills/$skill_slug/SKILL.md"
    if [[ ! -f "$skill_file" ]]; then
        printf 'Missing OS skill for %s: %s\n' "$os_slug" "$skill_file" >&2
        exit 1
    fi

    IFS=',' read -ra command_names <<< "$aliases"
    for command_name in "${command_names[@]}"; do
        for exposed_name in "$command_name" "omg-$command_name"; do
            cat > "$claude_dir/$exposed_name.md" <<EOF
# /$exposed_name

Run $os_slug. Read and follow the complete operating contract in:

\`$skill_file\`

Use every referenced agent, protocol, schema, template, and verification step.
EOF
            cat > "$codex_dir/$exposed_name.md" <<EOF
# /$exposed_name

Run $os_slug. Read and follow the complete operating contract in:

\`$skill_file\`

Use every referenced agent, protocol, schema, template, and verification step.
EOF
            installed=$((installed + 2))
        done
    done
done < <(os_command_catalog)

printf 'Installed %s OmegaOS command files across Claude and Codex.\n' "$installed"
