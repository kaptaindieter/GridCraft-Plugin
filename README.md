# GridCraft-Plugin — Craft Suite Plugin for GridCraft

Eigenstaendiges Craft-Suite-Plugin fuer Vanilla GridCraft: Integrierter BYOK AI-Agent mit Tabellen-, Formel- & Modell-Commands, dynamischer Cloud- & Auto-Save-Sync (frei waehlbarer Ordner) und Aurora/Dark/Light Farbschema-System.

> **Wichtiger Hinweis zur Architektur:**
> Dieses Repository enthaelt **ausschliesslich das eigenstaendige Plugin** (die neuen Rust-Module, AI-Icons, den Hook-Patch und den Installer) und **nicht** den gesamten Vanilla-Quellcode von GridCraft. Jeder Nutzer laedt sich einfach die originale Vanilla-Version von [https://github.com/storytold/gridcraft](https://github.com/storytold/gridcraft) herunter und installiert anschliessend dieses Plugin darueber.

---

## Features des Plugins

### 1. BYOK AI-Agent & Command-Bridge (`artcraft_agent.rs`)
- **Bring Your Own Key (BYOK)**: Jeder Nutzer kann in der AI-Sidebar ueber **`+ API`** bzw. **`Manage APIs`** seine eigenen API-Keys und Endpunkte hinterlegen (OpenAI / GPT, Google Gemini, DeepSeek, Mistral, Qwen, lokales Ollama oder jeder OpenAI-kompatible Server).
- **Keine hartcodierten API-Keys**: Alle Zugangsdaten werden ausschliesslich lokal im jeweiligen Benutzerprofil (`~/.craft-suite/shared-ai-config.json` bzw. `%APPDATA%\CraftSuite\shared-ai-config.json`) gespeichert.
- **Direkte App-Steuerung ueber Commands**:
  - Unterstuetzte Commands: `sheet.insertStyledTable`, `sheet.buildFinancialModel`, `cell.setValue`, `cell.setFormula`, `range.format`, `get_sheet_summary`, `save_workbook`

### 2. Dynamisches Cloud- & Auto-Save-Sync (`artcraft_drive.rs`)
- **Frei waehlbarer Sync-Ordner fuer jeden Endnutzer**: Ueber einen Klick auf das Cloud-Status-Badge in der oberen rechten Aktionsleiste oeffnet sich das Konfigurationsfenster **`☁️ Cloud & Auto-Save Sync`**.
- **Anbieter & Ordner selbst waehlen**: Unterstuetzt Google Drive, OneDrive, Nextcloud, Dropbox oder jeden beliebigen lokalen Ordner ueber den nativen Ordner-Auswahldialog (**`📂 Choose Folder`**).
- **Lokale Persistenz**: Die gewaehlte Pfad-Konfiguration wird lokal unter `~/.craft-suite/cloud-config.json` gespeichert (keine fremden Nutzerpfade im Code).

### 3. Farbschema-System (`Aurora`, `Dark`, `Light`) & UI-Design (`artcraft_theme.rs`)
- **Neues Farbschema-System**: Erweitert GridCraft um den kosmischen **Aurora**-Modus (inkl. Regenbogen-Akzentlinie) zusaetzlich zu **Dark** und **Light**.
- **Einheitliche Aktionsleiste & Theme-Switcher**: Standardisierte 28px Top-Right Action Bar (`✓ Auto-Save`, `● Cloud-Status`, `✨ AI Agent`) sowie Theme-Umschalter in der Fusszeile.

---

## Kompatibilitaet mit der Vanilla-Version

- **Originales Upstream-Repository**: [https://github.com/storytold/gridcraft](https://github.com/storytold/gridcraft)
- **Getesteter Vanilla-Basis-Commit**: `df137b8ee8cd68503681f759f59b4e30b2e0b048` (*"xlsx: repair repeated sheet names through a set of assigned names (#3211)"*)

---

## Installation (Schritt fuer Schritt)

### Voraussetzungen
- **Git** und **Python 3.8+** (oder PowerShell unter Windows)
- **Rust Toolchain** (`cargo`, Rust 1.80+)

### 1. Originale Vanilla-Version herunterladen
```bash
git clone https://github.com/storytold/gridcraft.git
cd gridcraft
# Optional: Exakten getesteten Vanilla-Stand auschecken
git checkout df137b8ee8cd68503681f759f59b4e30b2e0b048
cd ..
```

### 2. Dieses Plugin herunterladen & installieren
```bash
git clone https://github.com/kaptaindieter/GridCraft-Plugin.git
cd GridCraft-Plugin

# Plugin auf die Vanilla-Version anwenden (und optional direkt kompilieren):
python install_plugin.py ../gridcraft --build
```

Oder unter Windows per PowerShell:
```powershell
.\install_plugin.ps1 -TargetDir ..\gridcraft -Build
```

Die fertige ausfuehrbare Datei liegt anschliessend unter:
`../gridcraft/target/release/gridcraft.exe` (bzw. `target/release/gridcraft` unter Linux/macOS).

---

## Deinstallation (Vanilla-Zustand wiederherstellen)

Falls du das Vanilla-Repository wieder in den 100% originalen Zustand zurueckversetzen moechtest:

```bash
python uninstall_plugin.py ../gridcraft
```

Oder per PowerShell:
```powershell
.\uninstall_plugin.ps1 -TargetDir ..\gridcraft
```

---

## Paketstruktur des Plugins

- `plugin/src/artcraft_agent.rs`
- `plugin/src/artcraft_mcp.rs`
- `plugin/src/artcraft_drive.rs`
- `plugin/src/artcraft_theme.rs`
- `plugin/assets/ai_icons/` — Icons fuer die KI-Modell-Anbieter (OpenAI, Gemini, DeepSeek, Llama, Mistral, Qwen, AI)
- `patches/vanilla-hooks.patch` — Minimaler Integrations-Patch fuer die Vanilla-Dateien:
- `apps/gridcraft-web/index.html`
- `crates/ui-egui/Cargo.toml`
- `crates/ui-egui/src/app.rs`
- `crates/ui-egui/src/app/commands.rs`
- `crates/ui-egui/src/app/prefs.rs`
- `crates/ui-egui/src/backstage.rs`
- `crates/ui-egui/src/backstage/options.rs`
- `crates/ui-egui/src/dialogs/about.rs`
- `crates/ui-egui/src/lib.rs`
- `crates/ui-egui/src/ribbon.rs`
- `crates/ui-egui/src/ribbon/title_bar.rs`
- `crates/ui-egui/src/ribbon/view.rs`
- `crates/ui-egui/src/tabs.rs`
- `crates/ui-egui/src/theme.rs`
- `crates/ui-egui/src/theme/font_setup.rs`
- `config/cloud-config.example.json` — Beispiel-Vorlage fuer die Endnutzer-Cloud-Konfiguration
- `config/shared-ai-config.example.json` — Beispiel-Vorlage fuer die eigene BYOK-API-Konfiguration
- `config/UPSTREAM_VERSION.json` — Versions- & Commit-Metadaten der Vanilla-Basis
- `install_plugin.py` / `install_plugin.ps1` — Automatischer Plugin-Installer
- `uninstall_plugin.py` / `uninstall_plugin.ps1` — Automatischer Plugin-Uninstaller

---

## Lizenz

MIT License — siehe [LICENSE](LICENSE).
