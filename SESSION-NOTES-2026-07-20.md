# Session Notes — July 20, 2026

## Issues Solved

### 1. WSL Using Windows npm Instead of WSL npm

**Problem:** Running `npm` in WSL was using the Windows-installed npm (`/mnt/c/Users/gfg285/AppData/Roaming/npm`) instead of the WSL-native one (`/usr/bin/npm`). This happened because WSL automatically appends Windows paths to `$PATH` by default.

**Solution:**
- Edited `/etc/wsl.conf` and added:
  ```ini
  [interop]
  appendWindowsPath = false
  ```
- Restarted WSL with `wsl --shutdown` from PowerShell
- After restart, `$PATH` only contains Linux paths — no more `/mnt/c/...` entries

---

### 2. "Source file couldn't be found" Error When Converting DOCX to PDF

**Problem:** The web app's convert API (`web/src/app/api/convert/route.ts`) had two issues:

1. **Hardcoded Windows path for `flip` binary** (line 8):
   ```ts
   const FLIP_BIN = process.env.FLIP_BIN || "C:\\Users\\Talal\\.cargo\\bin\\flip.exe";
   ```
   This Windows path doesn't resolve in WSL.

2. **LibreOffice search prioritized Windows paths** — `findLibreOffice()` checked Windows paths first, which don't exist in WSL.

3. **Missing `libreoffice-writer` package** — Only `libreoffice-core` was installed, which has no document conversion filters.

**Solution:**

1. Installed the `flip` binary in WSL:
   ```bash
   cargo install --path crates/flip-cli
   ```

2. Fixed `route.ts`:
   - Changed `FLIP_BIN` default from the hardcoded Windows path to `"flip"` (resolves from system PATH)
   - Reordered `findLibreOffice()` to check Linux paths (`/usr/bin/libreoffice`, `/usr/bin/soffice`, `/usr/local/bin/libreoffice`) first, Windows paths last

3. Installed `libreoffice-writer`:
   ```bash
   sudo apt install -y libreoffice-writer
   ```

4. Installed `gh` CLI for GitHub authentication:
   ```bash
   sudo apt install -y gh
   gh auth login
   ```

---

## Files Modified

| File | Change |
|------|--------|
| `/etc/wsl.conf` | Added `[interop] appendWindowsPath = false` |
| `web/src/app/api/convert/route.ts` | Fixed `FLIP_BIN` path + reordered LibreOffice search |
| `web/next.config.ts` | Added `output: "standalone"` for Docker |
| `.dockerignore` | Created — excludes build artifacts, node_modules, etc. |
| `Dockerfile` | Created — multi-stage build (Rust + Node.js + LibreOffice) |

## Commits

- `4118fe2` — Fix WSL compatibility: use system PATH for flip and prioritize Linux paths for LibreOffice

## Pending (Next Session)

1. Install Docker in WSL
2. Build Docker image: `docker build -t flip-web .`
3. Test: `docker run -p 3000:3000 flip-web`
