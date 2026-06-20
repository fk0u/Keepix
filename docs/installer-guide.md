# Custom Installer & Production Signing Guide

Keepix uses a highly tailored installation sequence to match its premium branding. Instead of standard generic setup wizards, Keepix features custom-branded **NSIS (Nullsoft Scriptable Install System)** setups for Windows, and notarized DMG bundles for macOS.

---

## 📦 Installer Customization (NSIS)

Windows installers are configured via the `"nsis"` section in `src-tauri/tauri.conf.json`.

### 1. Branding Assets
The installer embeds specific high-fidelity assets during compilation:
- **`nsis-sidebar.bmp`**: A $164 \times 314$ vertical banner rendered on the welcome and finish screens.
- **`nsis-header.bmp`**: A $150 \times 57$ horizontal icon rendered in the upper-right corner during file extraction.
- **`icon.ico`**: The application's desktop and shell icon ($256 \times 256$ multi-size ICO).

> [!TIP]
> Keepix includes a custom Python utility [generate_installer_assets.py](file:///d:/Project/Keepix/generate_installer_assets.py) to programmatically draw and export these BMP headers and sidebar banners with appropriate bit depth and alignment. If you update the icons, run the script to refresh the assets:
> ```bash
> python generate_installer_assets.py
> ```

### 2. Svelte/Tauri Configuration
Our `tauri.conf.json` defines the installer behaviors:
```json
"bundle": {
  "active": true,
  "targets": ["nsis", "msi"],
  "identifier": "com.keepix.app",
  "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.icns", "icons/icon.ico"],
  "resources": [],
  "windows": {
    "nsis": {
      "headerImage": "icons/nsis-header.bmp",
      "sidebarImage": "icons/nsis-sidebar.bmp",
      "installerIcon": "icons/icon.ico",
      "installMode": "currentUser",
      "languages": ["English", "Indonesian"],
      "oneClick": false,
      "allowToChangeInstallationDirectory": true
    }
  }
}
```

- **`installMode: "currentUser"`**: Installs the app inside the user's local app directory. This prevents User Account Control (UAC) administrative permission popups, allowing standard users to install Keepix.
- **`oneClick: false`**: Displays the welcome page, directory selection, and components list, giving the user full autonomy.
- **`languages: ["English", "Indonesian"]`**: Configures multi-language installation dialogues. NSIS automatically detects the user's OS language and renders the installer text accordingly.

---

## 🔑 Production Code Signing

Unsigned applications trigger Windows SmartScreen warnings ("Windows protected your PC") or macOS Gatekeeper blocks ("App is damaged and cannot be opened"). Signing is critical for professional distribution.

### 1. Windows Code Signing
To sign the Windows executable (`.exe` and `.msi`):
- Acquire a **sectigo** or **DigiCert** EV Code Signing Certificate.
- Use Tauri's built-in hooks or run `signtool.exe` manually during the build pipeline:
  ```powershell
  SignTool sign /f path\to\certificate.pfx /p "YourPassword" /tr http://timestamp.digicert.com /td sha256 /fd sha256 src-tauri\target\release\bundle\nsis\Keepix_setup.exe
  ```
- Alternatively, define the signing certificate environment variables in your GitHub Actions / GitLab CI runner, and Tauri will sign the bundle automatically:
  - `TAURI_SIGNING_PRIVATE_KEY`: Path/content of the private key.
  - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: Password for the key.

### 2. macOS Code Signing & Notarization
To distribute Keepix on macOS:
- Install Xcode Command Line Tools.
- Establish environment variables for your Apple Developer ID:
  - `APPLE_SIGNING_IDENTITY`: Apple Developer ID Application certificate name (e.g., `Developer ID Application: Your Studio Name (ID)`).
  - `APPLE_ID`: Your Apple Developer account email.
  - `APPLE_PASSWORD`: App-specific password generated on appleid.apple.com.
  - `APPLE_TEAM_ID`: 10-character Developer Team ID.
- Run `npm run tauri build`. Tauri will compile, sign, upload to Apple's notary service, wait for approval, and staple the ticket onto the resulting DMG package.

---

## 🚀 Release Compilation Commands

To compile the signed, release-ready installer:

1. Clean previous build artifacts:
   ```bash
   cargo clean
   ```
2. Build the production package:
   ```bash
   npm run tauri build
   ```
3. Locate the installers:
   - **Windows setup**: `src-tauri/target/release/bundle/nsis/Keepix_4.x.x_x64-setup.exe`
   - **Windows MSI**: `src-tauri/target/release/bundle/msi/Keepix_4.x.x_x64.msi`
   - **macOS DMG**: `src-tauri/target/release/bundle/dmg/Keepix_4.x.x_x64.dmg`
