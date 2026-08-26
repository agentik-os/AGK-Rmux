# Installation

Copy the folder to:

```text
omega-os/os/strategy-portfolio/
```

Register:
1. `config/os.yaml`
2. `config/router.json`
3. `system/SYSTEM_PROMPT.md`
4. required agents and skills
5. schemas in the storage adapter

Validate:

```bash
python runtime/os_runtime.py validate
python runtime/os_runtime.py info
```

No external package is required by the reference runtime.
