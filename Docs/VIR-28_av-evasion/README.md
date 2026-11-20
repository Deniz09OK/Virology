# VIR-28 — AV Evasion

**MITRE :** T1562.001 — Impair Defenses | T1027 — Obfuscated Files or Information  
**Priorité :** Highest  
**Statut :** Terminé ✅

---

## Objectif

L'implant Rust doit s'exécuter sans déclencher Windows Defender ni Sysmon
sur la VM cible Windows 11 25H2 (Defender activé, niveau de protection standard).

---

## Étape 1 — Détection statique

### Obfuscation des strings (`obfstr`)

Windows Defender scanne les chaînes de caractères dans le binaire (`.rdata`).
Le crate `obfstr` chiffre chaque string littérale à la **compilation** via XOR
et la déchiffre en mémoire seulement au moment de l'utilisation.

```rust
let dll = hide!("kernel32.dll");   // aucune string visible dans le binaire
```

**Résultat :** les IOC textuels (noms de DLL, chemins, URLs) disparaissent du binaire.

### Profil de compilation release

```toml
[profile.release]
opt-level    = "z"       # binaire le plus compact possible
lto          = true      # Link-Time Optimization — réduit la surface analysable
codegen-units = 1
panic        = "abort"   # supprime le runtime de panic (réduit la taille)
strip        = true      # supprime les symboles de debug
```

---

## Étape 2 — Contournement des hooks EDR (API Hashing)

### Problème : hooks userland

Les EDR et AV (Defender inclus) placent des **hooks** dans les DLL système
(`kernel32.dll`, `ntdll.dll`) en réécrivant les premiers octets des fonctions
sensibles (`VirtualAlloc`, `CreateThread`, etc.) pour intercepter les appels.

Un implant qui importe ces fonctions normalement est immédiatement détecté
par analyse de l'IAT (Import Address Table).

### Solution : résolution dynamique par hash

Plutôt que d'importer les fonctions par leur nom, on **walk l'Export Address
Table** de la DLL en mémoire et on compare les noms exportés à un hash FNV-1a
calculé à la compilation.

```
GetModuleHandleA("kernel32.dll")
        │
        ▼
DOS header → e_lfanew → NT headers
        │
        ▼
DataDirectory[0] = Export Table RVA
        │
        ▼
IMAGE_EXPORT_DIRECTORY
  AddressOfNames      → liste des noms exportés
  AddressOfFunctions  → liste des RVA de fonctions
  AddressOfOrdinals   → correspondance nom → fonction
        │
        ▼
Comparer fnv1a(nom) == H_VIRTUAL_ALLOC
        │ match
        ▼
Récupérer l'adresse de la vraie fonction en mémoire
(en-dessous du hook EDR)
```

**Résultat :** l'IAT ne contient aucun nom de fonction sensible.
Les hooks EDR dans les stubs DLL sont contournés car on résout directement
l'adresse en mémoire.

---

## Étape 3 — Compilation et exécution

### Compilation (depuis le host Linux — cross-compile)

```bash
# Installer la target Windows et cargo-xwin (une seule fois)
rustup target add x86_64-pc-windows-msvc
cargo install cargo-xwin

# Compiler le crate implant en release pour Windows
cargo xwin build --release --target x86_64-pc-windows-msvc -p implant

# Binaire produit :
# target/x86_64-pc-windows-msvc/release/implant.exe
```

> **Note :** `cargo-xwin` télécharge automatiquement les headers MSVC (cl.exe, CRT).
> Nécessite un accès à `aka.ms` (réseau ouvert, pas derriere proxy restrictif).

### Compilation alternative (depuis Windows directement)

```powershell
# Prérequis : Visual Studio Build Tools ou MSVC installé
cargo build --release --target x86_64-pc-windows-msvc -p implant
```

### Transfert sur la VM cible

```bash
# Depuis le host (192.168.56.112) vers la VM (192.168.56.111)
# Via partage dossier VirtualBox ou SCP si SSH activé
scp target/x86_64-pc-windows-msvc/release/implant.exe user@192.168.56.111:C:\Temp\
```

### Exécution sur la VM

```powershell
# Sur la VM Windows 11 — terminal standard (Medium Integrity)
C:\Temp\implant.exe
```

**Comportements attendus :**

| Résultat | Signification |
|----------|---------------|
| Processus exit 0 silencieux | Tous les EAT walks réussis, évasion OK |
| Alerte Defender avant exécution | Détection statique — obfstr ou profil release insuffisant |
| Panic `EAT walk failed: VirtualAlloc` | Bug dans `resolve()` — offset PE incorrect |
| Alerte Defender pendant exécution | Détection comportementale — API hashing insuffisant |

### Vérification de l'IAT (analyse statique post-compilation)

```bash
# Sur le host Linux — vérifier qu'aucune API sensible n'est importée
strings target/x86_64-pc-windows-msvc/release/implant.exe | grep -iE 'virtualalloc|createthread|openprocess|loadlibrary'
# Résultat attendu : aucune ligne (strings chiffrées, IAT vide pour ces API)
```

---

## Résultats de test

> Testé sur Windows 11 (VM), Defender activé, niveau de protection standard.
> Build : `cargo build --release --target x86_64-pc-windows-msvc`

| Technique | Résultat | Alerte Defender | Event ID Sysmon observé |
|-----------|----------|-----------------|-------------------------|
| obfstr (strings XOR) | ✅ Binaire sans strings lisibles | ✅ Aucune alerte | Aucun |
| API Hashing (EAT walk) | ✅ 4/4 résolutions réussies (exit 0 silencieux) | ✅ Aucune alerte | Aucun |
| Profil release (strip+lto) | ✅ Symboles supprimés, LTO actif | ✅ Aucune alerte | Aucun |

**Verdict global :** ✅ Non détecté — implant compilé et exécuté sans alerte Defender ni event Sysmon.

---

## Structure des fichiers

```
implant/src/evasion/
├── mod.rs            # exports du module
├── api_hashing.rs    # fnv1a() + resolve() — walk EAT
└── obfuscation.rs    # macro hide!() — obfstr wrapper
```

---

## Note Blue Team — Comment détecter

### Event IDs clés

| Event ID | Source | Signal |
|----------|--------|--------|
| Sysmon 1 | Process Create | Exécution d'un binaire inconnu depuis `%APPDATA%` |
| Sysmon 3 | Network Connection | Connexion HTTPS sortante vers IP non catégorisée |
| Sysmon 7 | Image Loaded | Chargement de DLL sans entrée IAT correspondante |
| Sysmon 10 | Process Access | Accès à la mémoire de `kernel32.dll` / `ntdll.dll` |
| Windows 4698 | Security | Création d'une Scheduled Task par un process inconnu |

### Règles Sigma

#### Détection EAT Walk

```yaml
title: Suspicious Direct Memory Access to System DLL
id: vir28-sigma-001
status: experimental
description: |
  Détecte un process accédant directement à la mémoire de DLLs système
  sans passer par GetProcAddress — pattern typique d'un EAT walk (API hashing).
references:
  - https://attack.mitre.org/techniques/T1027/
logsource:
  product: windows
  service: sysmon
detection:
  selection:
    EventID: 10
    TargetImage|endswith:
      - '\kernel32.dll'
      - '\ntdll.dll'
    CallTrace|contains: 'UNKNOWN'
  filter_legit:
    SourceImage|startswith:
      - 'C:\Windows\'
      - 'C:\Program Files\'
  condition: selection and not filter_legit
falsepositives:
  - Outils de sécurité légitimes (debuggers, AV)
level: medium
tags:
  - attack.defense_evasion
  - attack.t1027
```

#### Détection connexion réseau depuis binaire non signé

```yaml
title: Unsigned Binary HTTPS Outbound Connection
id: vir28-sigma-002
status: experimental
description: |
  Connexion HTTPS sortante depuis un binaire non signé situé hors
  des répertoires système standard.
logsource:
  product: windows
  service: sysmon
detection:
  selection:
    EventID: 3
    Initiated: 'true'
    DestinationPort: 443
  filter_signed:
    Image|startswith:
      - 'C:\Windows\'
      - 'C:\Program Files\'
      - 'C:\Program Files (x86)\'
  condition: selection and not filter_signed
falsepositives:
  - Applications utilisateur non signées légitimes
level: medium
tags:
  - attack.command_and_control
  - attack.t1071
```

### Règle YARA — détection FNV-1a dans le binaire

```yara
rule AV_Evasion_FNV1A_API_Hashing {
    meta:
        description = "Détecte l'usage du hash FNV-1a pour résolution dynamique d'API"
        mitre       = "T1027"
    strings:
        $fnv_prime  = { 93 C1 F7 00 }   // 16777619 en little-endian
        $fnv_offset = { 61 C8 11 7C }   // 2166136261 en little-endian
    condition:
        uint16(0) == 0x5A4D and all of them
}
```

### IOC réseau

- Connexion HTTPS port 443 vers `192.168.56.112` (host-only, lab uniquement)
- Pas de SNI légitime (certificat auto-signé, CN=localhost)
- Intervalle de beacon régulier : ~5s ± 2s (détectable par analyse de flux réseau)
