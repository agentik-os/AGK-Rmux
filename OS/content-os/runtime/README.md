# Reference Runtime

This standard-library-only CLI proves that the package is self-describing and integrity-checkable.

```bash
python runtime/os_runtime.py info
python runtime/os_runtime.py route "/content"
python runtime/os_runtime.py event note '{"example": true}'
python runtime/os_runtime.py validate
```

It is not a production database, LLM adapter or security layer.
