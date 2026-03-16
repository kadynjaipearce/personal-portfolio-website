# Web Interface Guidelines – Audit

Grouped by file. Terse findings (file:line).

## templates/base.html

- base.html:2 - no skip link to main content
- base.html:20 - Inter is generic; consider distinctive display + body pair
- base.html:106-108 - scroll-behavior: smooth; add prefers-reduced-motion: reduce variant
- base.html:75-79 - typing cursor animation; add @media (prefers-reduced-motion: reduce) to disable
- base.html:44-56 - keyframes animations; add reduced-motion variant or disable when prefers-reduced-motion

## templates/components/nav.html

- nav.html:23-27 - GitHub icon link: decorative SVG missing aria-hidden="true"
- nav.html:31-35 - mobile menu button is icon-only: missing aria-label (e.g. "Open menu")
- nav.html:36-37 - SVG inside button: aria-hidden="true"
- nav.html:46-54 - x-show transition: ensure reduced-motion respected (Alpine)

## templates/components/footer.html

- footer.html:33-36, 39-42, 47-49 - decorative SVGs next to links: add aria-hidden="true" on SVGs

## templates/pages/contact.html

- contact.html:45,52,59,66 - focus:outline-none (has ring replacement); add focus-visible:ring-2 for keyboard
- contact.html:46 - input name: add autocomplete="name"
- contact.html:52 - input email: add autocomplete="email"
- contact.html:67 - placeholder "Your message..." → "Your message…"
- contact.html:73-77 - submit button: SVG decorative, add aria-hidden="true"
- contact.html:18-19 - success state: consider aria-live="polite" for dynamic message

## templates/admin/layout.html

- admin/layout.html:49-94 - nav links with inline SVG: decorative SVGs need aria-hidden="true"
- admin/layout.html:115 - delete button (messages/view): icon-only, need aria-label
- admin/layout.html: no skip link to main (admin has sidebar; main content should be skip target)

## templates/admin/messages/view.html

- view.html:15-19 - icon-only delete button: missing aria-label="Delete message"
- view.html:36-41 - Reply button SVG: aria-hidden="true"

## templates/admin/projects/form.html (and other admin forms)

- All inputs: add focus-visible:ring-2 alongside focus:ring-2
- Placeholders: use "…" not "..." where applicable
- Optional: autocomplete on name/url fields

## templates/pages/home.html

- home.html:38-42 - CTA buttons: SVG arrows decorative, aria-hidden="true"
- home.html:74-78 - GitHub link SVG: aria-hidden="true"
- home.html:54 - counter: tabular-nums already (good)

## General

- Heading hierarchy: ensure single h1 per page, then h2/h3 in order (audit each page).
- All icon-only actions: aria-label.
- All decorative icons: aria-hidden="true".
- Forms: autocomplete, placeholder ellipsis "…", labels associated (for/id).
- Animation: prefer transform/opacity; list properties not transition: all; honor prefers-reduced-motion.
