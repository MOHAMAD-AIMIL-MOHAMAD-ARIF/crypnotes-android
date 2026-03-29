.PHONY: generate-kotlin-bindings

generate-kotlin-bindings:
	powershell -ExecutionPolicy Bypass -File scripts/generate-kotlin-bindings.ps1
