# Bug Reports — s0P0wn3d

Ce dossier centralise les rapports de bugs identifiés pendant le développement et les tests du C2.

---

## Template de rapport de bug

Copier le bloc ci-dessous pour chaque nouveau bug.

```markdown
## [BUG] <Titre court et explicite>

**Ticket Jira :** VIR-XX  
**Date :** YYYY-MM-DD  
**Rapporté par :** <Prénom Nom>  
**Sévérité :** Critique / Haute / Moyenne / Basse

### Description

<Description claire du problème observé.>

### Étapes pour reproduire

1. ...
2. ...
3. ...

### Comportement attendu

<Ce qui aurait dû se passer.>

### Comportement observé

<Ce qui s'est passé à la place.>

### Environnement

| Paramètre     | Valeur                        |
|---------------|-------------------------------|
| Composant     | implant / c2_server / commun  |
| OS cible      | Windows 11 25H2 / Host Linux  |
| Rust toolchain| ex: 1.81.0-x86_64-pc-windows-msvc |
| Réseau        | Host-Only vboxnet0            |

### Logs / Captures

<Coller les logs pertinents ou captures d'écran ici.>

### Cause identifiée

<Si connue, décrire la cause racine.>

### Fix appliqué

<Décrire la correction apportée et le commit associé (`fix(VIR-XX): ...`).>

### Statut

- [ ] Ouvert
- [ ] En cours
- [ ] Résolu
- [ ] Vérifié
```

---

## Conventions

- Nommer le fichier du rapport : `VIR-XX-<slug>.md` (ex: `VIR-48-beacon-reconnect.md`)
- Créer un ticket Jira associé avant d'ouvrir le rapport
- Le commit de fix suit le format : `fix(VIR-XX): <description>`
- Lier le rapport au ticket Jira dans le champ **Development**

## Sévérités

| Niveau    | Définition                                                        |
|-----------|-------------------------------------------------------------------|
| Critique  | Bloque totalement une fonctionnalité core (beacon, chiffrement)   |
| Haute     | Fonctionnalité dégradée, contournement difficile                  |
| Moyenne   | Impact partiel, contournement possible                            |
| Basse     | Cosmétique ou mineur, pas de blocage                              |
