---
version: alpha
name: Nagare Product UI
description: Source-derived design system for Nagare desktop work-management screens, extracted from crates/nagare-cli/src/ui_assets.rs and docs/design.md.
colors:
  background: "#f8fafc"
  surface: "#ffffff"
  surfaceSubtle: "#f1f5f9"
  surfaceTint: "#fbfdff"
  surfaceSheen: "#f8fbff"
  text: "#020617"
  muted: "#475569"
  border: "#e2e8f0"
  primary: "#4338ca"
  primaryDeep: "#312e81"
  primaryBright: "#2563eb"
  primarySoft: "#eef2ff"
  flowCyan: "#06b6d4"
  focus: "#a5b4fc"
  success: "#047857"
  successSoft: "#ecfdf5"
  warning: "#b45309"
  warningSoft: "#fffbeb"
  danger: "#b91c1c"
  dangerSoft: "#fef2f2"
typography:
  family: 'Inter, "Yu Gothic UI", Meiryo, Arial, sans-serif'
  pageTitle:
    fontSize: 24px
    fontWeight: 700
    lineHeight: 1.25
  sectionTitle:
    fontSize: 17px
    fontWeight: 700
    lineHeight: 1.35
  body:
    fontSize: 14px
    fontWeight: 400
    lineHeight: 1.45
  dense:
    fontSize: 12px
    fontWeight: 600
    lineHeight: 1.45
spacing:
  xs: 6px
  sm: 8px
  md: 12px
  lg: 18px
  xl: 24px
rounded:
  control: 7px
  panel: 8px
  badge: 12px
components:
  buttonPrimary:
    backgroundColor: "{colors.primaryDeep}"
    textColor: "#ffffff"
    rounded: "{rounded.control}"
    height: 32px
  buttonSecondary:
    backgroundColor: "{colors.surfaceSubtle}"
    textColor: "{colors.primary}"
    rounded: "{rounded.control}"
    height: 32px
  panel:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.panel}"
    padding: "{spacing.lg}"
  premiumPanel:
    backgroundColor: "{colors.surfaceSheen}"
    rounded: "{rounded.panel}"
    padding: "{spacing.lg}"
  input:
    backgroundColor: "{colors.surfaceTint}"
    textColor: "{colors.text}"
    rounded: "{rounded.control}"
    height: 38px
  textMuted:
    textColor: "{colors.muted}"
    typography: "{typography.dense}"
  divider:
    backgroundColor: "{colors.border}"
    height: 1px
  focusHalo:
    backgroundColor: "{colors.focus}"
    rounded: "{rounded.control}"
  primaryGlow:
    backgroundColor: "{colors.primaryBright}"
    rounded: "{rounded.control}"
    height: 2px
  streamAccent:
    backgroundColor: "{colors.flowCyan}"
    rounded: "{rounded.control}"
    height: 2px
  badgeInfo:
    backgroundColor: "{colors.primarySoft}"
    textColor: "{colors.primary}"
  badgeSuccess:
    backgroundColor: "{colors.successSoft}"
    textColor: "{colors.success}"
  badgeWarning:
    backgroundColor: "{colors.warningSoft}"
    textColor: "{colors.warning}"
  badgeDanger:
    backgroundColor: "{colors.dangerSoft}"
    textColor: "{colors.danger}"
---

## Overview

Nagare is a quiet operational tool for moving Work Items forward. The visual system is source-derived from the current Rust-generated UI: light slate workspace, white panels, indigo primary actions, compact Japanese-readable typography, 7-8px radius, thin borders, and semantic status colors.

The system must keep Work Item state and next action more prominent than agent internals. It should feel precise and inspectable, not like a decorative AI dashboard.

## Visual Thesis

The refined direction is a premium operations instrument: polished slate, white glass, and a consistent left-to-right flow whose energy grows toward the right edge. It uses pale cyan, blue, and deep indigo light that recalls Nagare's logo and product name. Luxury comes from restraint, depth control, typography contrast, and directional light, not from ornamental gold or unrelated decorative gradients.

## Source Basis

- `crates/nagare-cli/src/ui_assets.rs`: CSS custom properties, component radius, button, badge, panel, filter, table, and responsive patterns.
- `docs/design.md`: product direction, Work Item-first hierarchy, status semantics, and Hachiware Labs visual posture.
- `logo.png`: blue/indigo identity signal. In SVG prototypes, use a simplified editable mark rather than embedding the raster logo unless the screen needs exact brand reproduction.

## Colors

Use color mostly for state, selection, and action:

- Indigo is for primary actions, active navigation, selected filters, focus, and running/in-progress state.
- Flow accent gradients should run left-to-right with the lighter color at the left and the stronger blue/indigo at the right, matching Nagare's meaning as Flow.
- Subtle white-to-blue panel gradients are allowed for primary work surfaces so the app does not feel flat, but their direction must stay left-to-right.
- Work history rows use the same left-to-right gradient direction: they start from a neutral working surface on the left and gain semantic tint toward the right. The right side should carry the visual energy of the flow, while text remains readable.
- Row outlines stay solid, thin, and neutral gray. Do not apply status-colored borders, gradient strokes, or gradient edge lines to repeated Work rows; the flow effect belongs to the row fill, while status identity belongs to icons and chips.
- Green is for passed, done, and successful completion.
- Amber is for user attention, questions, confirmation, warnings, and draft review states.
- Red is for failed, blocked, destructive, or recovery-required states.
- Slate and white carry the normal working surface.

Avoid saturated full-screen gradients, bokeh, gradient orbs, and generic purple-blue SaaS decoration. The product should still read clearly in a mostly neutral state.

## Typography

Use Inter when available, with `Yu Gothic UI`, Meiryo, Arial, and sans-serif fallbacks. Keep the scale compact:

- Page title: 24px / 700.
- Section title: 17px / 700.
- Body and table rows: 13-14px.
- Badges, metadata, and filter labels: 11-12px.

Letter spacing is always 0. Do not scale font size with viewport width.

## Layout

Desktop screens use a 200px left navigation rail and a dense content workspace with 26-32px page padding, matching the current source. Panels are bordered, not heavily shadowed. Use tables or compact list rows for repeated Work Items.

The Work list follows an F-pattern: page header, new request composer, filter strip, then rows sorted by action need. Filters are ordered project first, work-state checkboxes second, keyword search last.

## Components

Panels use white or white-blue gradient surfaces with `#e2e8f0` borders and 8px radius. Gradients run left-to-right across the whole screen family so the interface feels like work moving forward. Controls use 7px radius. Badges use 12px pill radius. Buttons are compact and direct; use icon plus label for primary commands where the icon improves scan speed.

Use elevation only to separate layers: the active composer, toast, and floating controls may cast a soft slate shadow; repeated Work rows should not cast row-level shadows. They rely on thin neutral-gray borders, rightward semantic tint, and state chips so the lower edge never reads as a directional gradient line.

Status chips for Work rows must show both icon and label:

- `要対応・確認`
- `要対応・質問`
- `要対応・回復`
- `処理中`
- `完了`

Row actions use the unified label `詳細`; state-specific decisions happen inside the Work detail screen.

## Iconography

Use minimal 1.8px outline icons, matching Lucide-style geometry: folder, search, plus, check circle, help circle, alert triangle, clock, list filter, inbox, bot, book, users, settings, and chevron. Icons are functional labels, not decoration.

For self-contained SVG prototypes, draw icons with editable `<path>`, `<circle>`, and `<line>` primitives inside `<symbol>` definitions.

## Microinteractions

Represent the following states in visual comps when relevant:

- Primary action hover/focus uses the focus ring color `#a5b4fc`.
- Work creation shows a transient success toast.
- Running rows show an in-progress badge and no user action besides `詳細`.
- Question, confirmation, and recovery rows are attention states and open the same Work detail surface.
- Destructive actions use danger styling and are not visually primary.

## Do's And Don'ts

- Do keep Work Item, state, answer/progress preview, and next action visible in each row.
- Do keep diagnostics out of the primary list.
- Do use semantic status colors consistently across list and detail.
- Do not introduce a separate Result Review visual language.
- Do not show runtime names or raw logs in the Work list.
- Do not make the list look like a generic marketing dashboard.
