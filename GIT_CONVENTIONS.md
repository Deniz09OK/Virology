# Git Conventions — s0P0wn3d / Virology

## Format des commits

```
<type>(VIR-XX): <description courte>
```

### Types

| Type | Usage |
|---|---|
| `feat` | Nouvelle fonctionnalité / nouveau module |
| `fix` | Correction de bug |
| `docs` | Documentation (Docs/, README, CLAUDE.md) |
| `refactor` | Refactoring sans changement de comportement |
| `test` | Ajout ou modification de tests |
| `chore` | Tâches de maintenance (deps, config, .gitignore) |
| `wip` | Travail en cours (ne pas merger sur main) |

### Exemples

```bash
feat(VIR-25): ajout de la boucle principale du beacon HTTPS
feat(VIR-26): squelette c2_server avec endpoint /cmd (axum)
fix(VIR-28): correction encodage stdout du module shell sous Windows
docs(VIR-25): ajout doc Docs/VIR-25_c2-setup/README.md
refactor(VIR-30): séparation du chargement TLS dans communication/https.rs
chore: ajout de certs/ au .gitignore
```

---

## Branches

| Convention | Exemple |
|---|---|
| `main` | branche stable — code testé uniquement |
| `feat/VIR-XX-<slug>` | `feat/VIR-25-beacon-loop` |
| `fix/VIR-XX-<slug>` | `fix/VIR-28-shell-encoding` |
| `docs/VIR-XX-<slug>` | `docs/VIR-25-setup` |

```bash
# Créer une branche pour un ticket
git checkout -b feat/VIR-25-beacon-loop

# Push et ouvrir une PR
git push -u origin feat/VIR-25-beacon-loop
```

---

## Règles de merge

- **Pas de push direct sur `main`** — passer par une PR
- **Minimum 1 review** avant de merger (si possible)
- **La PR doit référencer le ticket** : mentionner `VIR-XX` dans le titre ou la description
- **Squash merge** pour garder l'historique propre sur main
- **Supprimer la branche** après merge

---

## Quand commiter

- Commiter **par fonctionnalité logique**, pas en masse en fin de journée
- Un commit = une chose (un module, un fix, une doc)
- Quand un ticket est terminé : commiter le code **et** le dossier `Docs/VIR-XX_<slug>/`

```bash
# Exemple ticket terminé
git add implant/src/execution/shell.rs Docs/VIR-28_shell-module/
git commit -m "feat(VIR-28): ajout du module shell distant via pipes anonymes"
```

---

## Fichiers à ne jamais commiter

Déjà dans `.gitignore`, à ne jamais forcer avec `git add -f` :

```
certs/cert.pem
certs/key.pem
.env
*.key
__pycache__/
*.pyc
```

---

## Workflow complet pour un ticket

```bash
# 1. Créer la branche depuis main à jour
git checkout main && git pull
git checkout -b feat/VIR-25-beacon-loop

# 2. Coder + commiter au fil de l'eau
git add implant/src/communication/https.rs
git commit -m "feat(VIR-25): squelette de la boucle polling du beacon"

git add implant/src/communication/https.rs
git commit -m "feat(VIR-25): ajout requête HTTPS avec vérification du certificat"

# 3. Ticket terminé → ajouter la doc
mkdir -p Docs/VIR-25_c2-setup
# rédiger Docs/VIR-25_c2-setup/README.md
git add Docs/VIR-25_c2-setup/
git commit -m "docs(VIR-25): ajout doc setup avec note MITRE T1071"

# 4. Push + PR
git push -u origin feat/VIR-25-beacon-loop

# 5. Passer le ticket Jira en "Terminé"
# 6. Mettre à jour le statut dans CLAUDE.md (🔜 → ✅)
```
