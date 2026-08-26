from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

@dataclass
class AgentRequest:
    prompt: str
    cwd: Path
    timeout_seconds: int = 3600

@dataclass
class AgentResult:
    return_code: int
    summary: str
    stdout: str = ""
    stderr: str = ""

class CodingAgentAdapter(Protocol):
    async def execute(self, request: AgentRequest) -> AgentResult:
        ...
