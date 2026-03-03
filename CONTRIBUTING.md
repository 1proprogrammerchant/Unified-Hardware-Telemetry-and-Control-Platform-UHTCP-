# Contributing & Publishing

This repository contains the UHTCP project. To publish this repository to GitHub, follow these steps locally:

1. Create a new repository on GitHub and copy its HTTPS URL.
2. From the repository root run:

```bash
git init
git add .
git commit -m "chore: initial import"
git branch -M main
git remote add origin <GIT_URL>
git push -u origin main
```

3. After pushing, create a release or tag in the GitHub UI if desired.

CI is configured under `.github/workflows/ci.yml` and will run on pushes and pull requests to `main`.

If you want to sign commits or add a code of conduct or CLA, add the relevant files before pushing.
