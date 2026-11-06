# VIR-28 — AV Evasion

**MITRE :** T1562.001 — Impair Defenses | T1027 — Obfuscated Files or Information  
**Priorité :** Highest  
**Statut :** En cours

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
use crate::hide;   // macro définie dans evasion/obfuscation.rs

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

### Hash FNV-1a (compile-time)

```rust
pub const H_VIRTUAL_ALLOC: u32 = fnv1a(b"VirtualAlloc");  // 0x...
```

Le hash est une constante numérique dans le binaire — pas de string.

---

## Structure des fichiers

```
implant/src/evasion/
├── mod.rs            # exports du module
├── api_hashing.rs    # fnv1a() + resolve() — walk EAT
└── obfuscation.rs    # wrappers obfstr + macro hide!()
```

---

## Note Blue Team — Comment détecter

| Signal | Détail | Outil |
|--------|--------|-------|
| Accès direct à la mémoire des DLL | `ReadProcessMemory` sur `kernel32.dll` sans appel `GetProcAddress` | Sysmon Event ID 10 |
| Chargement de DLL sans import visible | Pas d'entrée IAT pour les fonctions appelées | PE analysis (PE-bear, CFF Explorer) |
| Scan des hash connus | Chercher les constantes FNV-1a dans le binaire | YARA rule sur les valeurs `2166136261` et `16777619` |
| Comportement réseau HTTPS vers IP non catégorisée | Connexion sortante sur port 443 vers `192.168.56.112` | Sysmon Event ID 3, Wireshark |
| Scheduled Task créée par un process inconnu | Persistance via tâche planifiée — nom imitant un service Microsoft | Sysmon Event ID 11, Windows Event 4698 |

### Règle YARA indicative

```yara
rule AV_Evasion_FNV1A {
    meta:
        description = "Détecte l'usage du hash FNV-1a pour résolution dynamique d'API"
    strings:
        $fnv_prime  = { 93 C1 F7 00 }  // 16777619 en little-endian
        $fnv_offset = { 61 C8 11 7C }  // 2166136261 en little-endian
    condition:
        all of them
}
```
