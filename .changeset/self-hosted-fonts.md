---
main: patch
---

# No font flash on the website's first paint

The website self-hosts Bricolage Grotesque and Fragment Mono (latin and latin-ext woff2 subsets, with their SIL OFL 1.1 licences) and preloads both from every page's head. First paint uses the brand fonts instead of flashing the system stack while the Google Fonts stylesheet and files arrive, and the third-party font round trips are gone.
