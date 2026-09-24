# VIR-27 — Mise en place gitflow

**Auteur :** Deniz Ok
**Date :** 2026-09-24
**MITRE :** N/A — ticket organisation / processus équipe

## Ce que ça fait

Ce ticket met en place les conventions Git du projet Virology : nommage des branches,
format des commits, règles de merge et workflow complet ticket → PR → merge.
Le fichier `GIT_CONVENTIONS.md` à la racine du repo est la référence équipe.

---

## Branches en place

| Branche | Rôle |
|---------|------|
| `main` | Code stable — merge uniquement via PR validée |
| `dev` | Branche d'intégration — toutes les PR ciblent dev |
| `feat/VIR-XX-<slug>` | Une branche par ticket feature |
| `fix/VIR-XX-<slug>` | Une branche par ticket correctif |

> Minimum requis par le sujet : **3 branches**. Le repo en compte actuellement 4+.

---

## Format des commits

```
<type>(VIR-XX): <description courte>
```

| Type | Usage |
|------|-------|
| `feat` | Nouvelle fonctionnalité / nouveau module |
| `fix` | Correction de bug |
| `docs` | Documentation |
| `chore` | Maintenance (deps, config, .gitignore) |
| `refactor` | Refactoring sans changement de comportement |
| `test` | Ajout ou modification de tests |
| `wip` | Travail en cours (ne pas merger sur main) |

### Exemples

```bash
feat(VIR-29): ajout boucle beacon HTTPS avec jitter
fix(VIR-28): correction encodage stdout module shell
docs(VIR-25): ajout architecture C2 schéma + MITRE mapping
chore: ajout certs/ au .gitignore
```

---

## Règles de merge

- **Pas de push direct sur `main` ou `dev`** — toujours via PR
- **Minimum 1 review** avant merge
- **La PR référence le ticket** : `VIR-XX` dans le titre ou la description
- **Squash merge** pour garder l'historique propre
- **Supprimer la branche** après merge

---

## Workflow complet

```bash
# 1. Partir de dev à jour
git checkout dev && git pull
git checkout -b feat/VIR-XX-slug

# 2. Coder + commiter au fil de l'eau
git add <fichiers>
git commit -m "feat(VIR-XX): description"

# 3. Ticket terminé → ajouter la doc
# rédiger Docs/feat-VIR-XX-slug/README.md
git add Docs/feat-VIR-XX-slug/
git commit -m "docs(VIR-XX): ajout doc ticket"

# 4. Push + ouvrir une PR sur dev
git push -u origin feat/VIR-XX-slug
gh pr create --base dev --title "feat(VIR-XX): description"

# 5. Après merge → passer le ticket Jira en "Terminé"
```

---

## Fichiers à ne jamais commiter

Déjà couverts dans `.gitignore` :

```
certs/cert.pem
certs/key.pem
*.key / *.pem
.env
__pycache__/ / *.pyc
```

---

## Comment tester

```bash
# Vérifier les branches existantes
git branch -a

# Vérifier le format des commits récents
git log --oneline -10

# Vérifier que main et dev sont protégées (GitHub)
gh api repos/Deniz09OK/Virology/branches/main | jq '.protected'
gh api repos/Deniz09OK/Virology/branches/dev  | jq '.protected'
```

---

## Blue team — détection

N/A — ticket organisationnel, pas de surface d'attaque.

---

*Ticket : VIR-27 | Projet : Virology | Promo MSc 2027 — Epitech Nancy*
