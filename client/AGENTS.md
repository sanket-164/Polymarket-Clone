<!-- BEGIN:nextjs-agent-rules -->

# This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` before writing any code. Heed deprecation notices.

<!-- END:nextjs-agent-rules -->

# Theme Guide: Midnight Trading

> **Purpose:** This document defines the visual design system for this Polymarket-style prediction market app. Every component, page, and UI element you (the AI agent) generate must strictly follow these rules to keep the product visually consistent. Do not invent new colors, fonts, or spacing values outside what is defined here.

---

## 1. Design Philosophy

**Theme name:** Midnight Trading
**Vibe:** Dark, premium, low-glare, data-first. Built for traders who stare at charts and order books for hours.

Why this theme works:

- Very premium look and feel
- Easy on the eyes during long sessions
- Charts/graphs pop against the dark background
- Optimized for trading-style interfaces (order books, price tickers, position cards)

Keep this in mind for every design decision: **legibility of numbers and charts always wins over decoration.**

---

## 2. Color Palette

| Token        | Hex       | Usage                                                               |
| ------------ | --------- | ------------------------------------------------------------------- |
| `background` | `#09090B` | Root app background (body, page canvas)                             |
| `surface`    | `#111113` | Section backgrounds, navbars, sidebars, modals, table headers       |
| `card`       | `#18181B` | Cards, panels, dropdowns, tooltips, popovers — anything "elevated"  |
| `border`     | `#27272A` | All borders, dividers, input outlines, table row separators         |
| `text`       | `#FAFAFA` | Primary text — headings, prices, key values                         |
| `secondary`  | `#A1A1AA` | Muted text — labels, timestamps, helper text, placeholder text      |
| `buy`        | `#16A34A` | "Yes" / Buy actions, positive % change, profit values, up-candles   |
| `sell`       | `#EF4444` | "No" / Sell actions, negative % change, loss values, down-candles   |
| `accent`     | `#3B82F6` | Primary CTAs, links, active states, focus rings, highlights, badges |

### Elevation logic

Think of the UI in layers, each one slightly lighter than the last:

```
background (#09090B)
  → surface (#111113)
      → card (#18181B)
          → hover/active state of card (lighten ~4-6% further, do not introduce a new token)
```

Never place a `card` directly on `background` without a `surface` layer in between if the page has multiple sections — this keeps depth consistent.

### Semantic color rules (strict)

- **Green (`buy` #16A34A)** = Buy, Yes, positive change, gains, confirmations. Never use for anything unrelated to a positive/affirmative action.
- **Red (`sell` #EF4444)** = Sell, No, negative change, losses, destructive actions (delete, cancel order). Never use red for warnings that aren't loss/negative-related — use `accent` or a neutral state instead.
- **Blue (`accent` #3B82F6)** = Primary interactive actions only (main CTA buttons, active tab, selected filter, links, focus outlines, chart highlight line). Do not overuse — it should stand out precisely because it's used sparingly.
- **Never use pure black (`#000000`) or pure white (`#FFFFFF`)** anywhere. Always use the tokens above.

### Tailwind config reference

When implementing in Tailwind, register these as theme extensions so the agent (and future devs) always use tokens, not raw hex codes:

```js
// tailwind.config.js
theme: {
  extend: {
    colors: {
      background: '#09090B',
      surface: '#111113',
      card: '#18181B',
      border: '#27272A',
      text: {
        DEFAULT: '#FAFAFA',
        secondary: '#A1A1AA',
      },
      buy: '#16A34A',
      sell: '#EF4444',
      accent: '#3B82F6',
    },
  },
}
```

Use classes like `bg-card`, `text-secondary`, `border-border`, `text-buy`, `bg-accent`, etc. — never hardcode hex values in components.

### CSS variables (if not using Tailwind tokens directly)

```css
:root {
  --background: #09090b;
  --surface: #111113;
  --card: #18181b;
  --border: #27272a;
  --text: #fafafa;
  --text-secondary: #a1a1aa;
  --buy: #16a34a;
  --sell: #ef4444;
  --accent: #3b82f6;
}
```

---

## 3. Typography

**Font stack (in order of preference):**

1. **Inter** — primary UI font (body text, labels, buttons, nav)
2. **Geist** — alternative/fallback, especially good for numeric/data-heavy UI (prices, tickers, tables)
3. **SF Pro** — fallback for Apple devices / system font fallback

```css
font-family:
  "Inter",
  "Geist",
  "SF Pro",
  -apple-system,
  BlinkMacSystemFont,
  sans-serif;
```

### Usage guidelines

- **Headings (`h1`–`h3`):** Inter, semibold/bold (600–700 weight), `text` color (`#FAFAFA`)
- **Body text:** Inter, regular (400 weight), `text` color
- **Numeric data (prices, percentages, order book, charts):** prefer **Geist** or a tabular-numeral variant for alignment — numbers must line up in columns, use `font-variant-numeric: tabular-nums`
- **Secondary/meta text (timestamps, labels, helper text):** `secondary` color (`#A1A1AA`), regular weight, smaller size (12–13px)
- **Buttons:** Inter, medium/semibold (500–600 weight)

### Type scale (suggested)

| Use case                | Size    | Weight | Color                |
| ----------------------- | ------- | ------ | -------------------- |
| Page title (h1)         | 28–32px | 700    | `text`               |
| Section heading (h2)    | 20–24px | 600    | `text`               |
| Card title (h3)         | 16–18px | 600    | `text`               |
| Body text               | 14–15px | 400    | `text`               |
| Secondary/meta text     | 12–13px | 400    | `secondary`          |
| Large price/stat number | 24–36px | 700    | `text` (or buy/sell) |
| Button label            | 14px    | 600    | context-dependent    |

---

## 4. Component Guidelines

### Cards (market cards, position cards)

- Background: `card` (`#18181B`)
- Border: `1px solid border` (`#27272A`)
- Border radius: `12–16px`
- Padding: `16–20px`
- On hover: slightly lighten background or brighten border, add subtle shadow — do not change to an unrelated color

### Buttons

- **Primary CTA (e.g. "Place Order", "Sign Up"):** `accent` background, white/`text`-colored label, no border
- **Buy button:** `buy` background, white text
- **Sell button:** `sell` background, white text
- **Secondary/ghost button:** transparent or `surface` background, `border` outline, `text` label
- **Disabled state:** reduce opacity to ~40%, no color change

### Inputs / Forms

- Background: `surface` or `card`
- Border: `1px solid border`, focus state → `1px solid accent` + subtle accent glow/ring
- Placeholder text: `secondary`
- Input text: `text`

### Tables / Order books

- Header row: `surface` background, `secondary` text, uppercase, small size
- Row divider: `border`
- Row hover: subtle `card`-toned highlight
- Positive values (price up, gains): `buy`
- Negative values (price down, losses): `sell`
- Neutral values: `text` or `secondary`

### Charts / Graphs

- Up candles / positive lines: `buy` (`#16A34A`)
- Down candles / negative lines: `sell` (`#EF4444`)
- Gridlines: `border` at low opacity (~30–40%)
- Axis labels: `secondary`
- Chart background: transparent or `surface` (never pure black)
- Highlighted/selected data point or crosshair: `accent`

### Badges / Tags

- "Yes" tag: `buy` background at low opacity (e.g. 15%) with `buy` text
- "No" tag: `sell` background at low opacity with `sell` text
- Neutral/category tag: `surface` or `card` background with `secondary` text, `border` outline

### Navigation (navbar/sidebar)

- Background: `surface`
- Active link/tab: `accent` text or underline, `text` for inactive but visible, `secondary` for inactive/muted
- Border separating nav from content: `border`

---

## 5. Spacing & Radius (recommended defaults)

- Base spacing unit: `4px` grid (use multiples of 4: 4, 8, 12, 16, 20, 24, 32...)
- Card/container radius: `12–16px`
- Button/input radius: `8px`
- Badge/pill radius: `999px` (fully rounded)
- Page max-width / content container: keep generous margins on desktop; dense, data-first layouts inside cards

---

## 6. Do's and Don'ts

**Do:**

- Always use the defined color tokens (`background`, `surface`, `card`, `border`, `text`, `secondary`, `buy`, `sell`, `accent`)
- Keep contrast high between `text`/`secondary` and their background layer for readability
- Use `buy`/`sell` colors consistently and only for their semantic meaning
- Keep numeric data aligned and easy to scan (tabular numerals, right-aligned numbers in tables)

**Don't:**

- Don't introduce new hex colors outside this palette
- Don't use pure black or pure white
- Don't use `sell` (red) for generic warnings/errors unrelated to losses — reserve it for financial negative/destructive meaning
- Don't overuse `accent` — it should be reserved for primary actions and highlights only
- Don't stack `card` directly on `background` without a `surface` transition layer in multi-section pages

---

## 7. Quick Reference Summary

```
Background:  #09090B
Surface:     #111113
Card:        #18181B
Border:      #27272A
Text:        #FAFAFA
Secondary:   #A1A1AA
Buy:         #16A34A
Sell:        #EF4444
Accent:      #3B82F6

Fonts: Inter → Geist → SF Pro
```
