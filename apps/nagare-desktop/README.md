# Nagare desktop

Tauri desktop shell for the Nagare UI.

The desktop UI implements the current flow described in
`docs/design-assets/prototype/` and calls `nagare-core` through typed Tauri
commands. The prototype remains the visual and flow reference; this package is
the runnable desktop implementation.

## Commands

```powershell
npm run dev --workspace @hachiware-labs/nagare-desktop
npm run build --workspace @hachiware-labs/nagare-desktop
npx playwright test tests/e2e/desktop-ui.spec.ts
npm run test:e2e:desktop-window
```

`test:e2e:desktop-window` launches the built Tauri executable through
`tauri-driver` and verifies the real desktop window with an isolated
`NAGARE_ROOT`. It uses `NAGARE_MSEDGEDRIVER` when provided; otherwise, on
Windows, it downloads the matching Edge WebDriver for the installed WebView2
runtime when the bundled driver is out of date. Set
`NAGARE_DESKTOP_E2E_STRICT=1` when a skip should fail CI.
