# Android Package Scaffolding

This directory defines the v1 package roots expected by the CrypNotes Android architecture.

- `com.crypnotes.app`
- `com.crypnotes.core.bridge`
- `com.crypnotes.core.data`
- `com.crypnotes.core.platform.security`
- `com.crypnotes.core.platform.media`
- `com.crypnotes.core.platform.notifications`
- `com.crypnotes.core.ui`
- `com.crypnotes.feature.notes`
- `com.crypnotes.feature.labels`
- `com.crypnotes.feature.reminders`
- `com.crypnotes.feature.vault`
- `com.crypnotes.feature.settings`

`PackageMarker.kt` files are placeholders so package roots are explicit before module wiring.

## Kotlin UniFFI bindings

Generate Kotlin bindings into `com.crypnotes.core.bridge` (`:core:bridge` module) by running:

```bash
make generate-kotlin-bindings
```

Equivalent direct command:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/generate-kotlin-bindings.ps1
```
