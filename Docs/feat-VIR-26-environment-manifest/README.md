# VIR-26 — Setup Environment Manifest Reproductible

**Auteur :** Deniz Ok
**Date :** 2026-09-24
**MITRE :** N/A — ticket infrastructure / reproductibilité

## Contexte

Ce ticket documente la procédure complète pour monter l'environnement de lab du projet Virology.
L'objectif est qu'un membre de l'équipe puisse recréer le lab from scratch en suivant ce README.

---

## Specs de la VM

| Paramètre           | Valeur                                                      |
|---------------------|-------------------------------------------------------------|
| Nom                 | virology-target                                             |
| OS                  | Windows 11 25H2 — `Win11_25H2_EnglishInternational_x64_v2.iso` (Windows Defender activé) |
| vCPU                | 2 minimum recommandé                                        |
| RAM                 | 4 Go minimum — 6 Go recommandé                              |
| Disque              | 60 Go (dynamique)                                           |
| Réseau              | Host-Only (vboxnet0) ou Réseau ponté selon contexte         |
| IP                  | 192.168.56.111 (fixe recommandée) ou via DHCP               |
| C2 host (attaquant) | 192.168.56.112 (host-only) ou IP du host sur réseau ponté   |

> **Note réseau** : Deux modes sont supportés selon le contexte :
> - **Host-Only (`vboxnet0`)** — recommandé pour un usage solo, réseau isolé indépendant du wifi/réseau école
> - **Réseau ponté** — recommandé en contexte groupe, la VM est visible sur le réseau local
>
> Le DHCP peut être activé dans les deux modes. L'IP fixe (`192.168.56.111`) est une **commodité** pour éviter de mettre à jour la config de l'implant à chaque redémarrage — elle n'est pas obligatoire.
> Si tu utilises le DHCP, adapte l'IP cible dans `implant/src/config.rs` après chaque attribution.

---

## 1. Créer la VM dans VirtualBox

### Option A — Interface graphique (recommandé)

1. Télécharger l'ISO Windows 11 25H2 :
   👉 https://www.microsoft.com/fr-fr/software-download/windows11
   Fichier : `Win11_25H2_EnglishInternational_x64_v2.iso`
   > Windows Defender doit rester **activé** — c'est une exigence du projet pour tester l'évasion AV.

2. Ouvrir VirtualBox → **Nouvelle**
3. Nom : `virology-target`, Type : `Microsoft Windows`, Version : `Windows 11 (64-bit)`
4. RAM : `4096 Mo minimum` (6144 Mo recommandé), vCPU : `2 minimum`
5. Disque : `60 Go`, format VDI, allocation dynamique
6. Réseau → **Adaptateur 1** → `Réseau hôte uniquement` → `vboxnet0`
7. Démarrer la VM et installer Windows depuis l'ISO

### Option B — Ligne de commande (optionnel)

```bash
VBoxManage createvm --name "virology-target" --ostype Windows11_64 --register

VBoxManage modifyvm "virology-target" \
  --memory 6144 \
  --cpus 2 \
  --nic1 hostonly \
  --hostonlyadapter1 vboxnet0

VBoxManage createhd \
  --filename "$HOME/VirtualBox VMs/virology-target/virology-target.vdi" \
  --size 61440

VBoxManage storagectl "virology-target" \
  --name "SATA Controller" --add sata --controller IntelAhci

VBoxManage storageattach "virology-target" \
  --storagectl "SATA Controller" \
  --port 0 --device 0 --type hdd \
  --medium "$HOME/VirtualBox VMs/virology-target/virology-target.vdi"

VBoxManage storageattach "virology-target" \
  --storagectl "SATA Controller" \
  --port 1 --device 0 --type dvddrive \
  --medium /chemin/vers/win11-enterprise.iso
```

---

## 2. Configurer le réseau

Deux modes au choix selon le contexte d'utilisation.

### Mode A — Host-Only (usage solo, réseau isolé)

#### Interface graphique

1. VirtualBox → **Fichier** → **Gestionnaire de réseau hôte**
2. Vérifier que `vboxnet0` existe avec :
   - IP : `192.168.56.1` · Masque : `255.255.255.0`
   - DHCP : désactivé (si IP fixe) ou activé (si DHCP)

#### CLI

```bash
VBoxManage hostonlyif create
VBoxManage hostonlyif ipconfig vboxnet0 --ip 192.168.56.1 --netmask 255.255.255.0
# Désactiver DHCP si IP fixe sur la VM
VBoxManage dhcpserver remove --netname HostInterfaceNetworking-vboxnet0 2>/dev/null || true
```

### Mode B — Réseau ponté (contexte groupe, VM visible sur le LAN)

#### Interface graphique

1. VM → **Configuration** → **Réseau** → **Adaptateur 1**
2. Mode d'accès réseau : `Accès par pont`
3. Interface : sélectionner la carte réseau active du host (Ethernet ou Wi-Fi)

#### CLI

```bash
VBoxManage modifyvm "virology-target" --nic1 bridged --bridgeadapter1 "Ethernet"
# Remplacer "Ethernet" par le nom exact de ton interface réseau active
```

> En mode ponté, la VM reçoit une IP du DHCP de ton réseau local. Relève-la avec `ipconfig` depuis la VM et mets à jour `implant/src/config.rs` en conséquence.

---

## 3. Assigner l'IP fixe dans la VM

Dans Windows (sur la VM) :

1. `Paramètres` → `Réseau et Internet` → `Ethernet` → `Modifier les options de l'adaptateur`
2. Clic droit sur l'interface → `Propriétés` → `Internet Protocol Version 4`
3. IP fixe :
   - Adresse : `192.168.56.111`
   - Masque : `255.255.255.0`
   - Passerelle : *(laisser vide)*
   - DNS : *(laisser vide)*

---

## 4. Snapshot — État de base

Prendre un snapshot **avant** tout test offensif pour pouvoir restaurer proprement.

### Option A — Interface graphique

1. VM démarrée (ou éteinte) → **Machine** → **Prendre un instantané**
2. Nom : `BASE — Windows 11 clean`

### Option B — CLI

```bash
VBoxManage snapshot "virology-target" take "BASE -- Windows 11 clean" \
  --description "Etat propre post-install"
```

---

## 5. Setup hôte (machine attaquante)

Sur ta machine (Windows) :

```bash
# Installer rustup si pas déjà fait
# https://rustup.rs

# Ajouter la cible Windows MSVC
rustup target add x86_64-pc-windows-msvc

# Vérifier
rustup target list --installed
```

---

## 6. Restaurer le snapshot

À répéter avant chaque session de test offensif pour repartir d'un état propre.

### Option A — Interface graphique

1. Éteindre la VM
2. Onglet **Instantanés** → Sélectionner `BASE — Windows 11 clean`
3. Clic droit → **Restaurer**

### Option B — CLI

```bash
VBoxManage controlvm "virology-target" poweroff 2>/dev/null || true
VBoxManage snapshot "virology-target" restore "BASE -- Windows 11 clean"
```

---

## 7. Checklist pré-démo

- [ ] VM `virology-target` démarrée
- [ ] VM accessible depuis le host (`ping <IP de la VM>`)
- [ ] IP de la VM connue et à jour dans `implant/src/config.rs`
- [ ] Snapshot `BASE` pris
- [ ] `rustup target list --installed` contient `x86_64-pc-windows-msvc`
- [ ] `cert.pem` et `key.pem` absents du repo (vérifier `.gitignore`)

---

## Sécurité

- Le beacon et le serveur tournent **uniquement** sur le réseau host-only (`192.168.56.0/24`) — jamais exposé sur le réseau réel.
- Ne jamais commiter `cert.pem`, `key.pem`, ou des credentials réels sur GitHub.
- Ce lab est réalisé dans un cadre pédagogique Epitech, sur un réseau fermé et consenti.

---

## Comment tester

```bash
# Depuis le host, vérifier la connectivité avec la VM
ping 192.168.56.111

# Vérifier que le serveur C2 est joignable depuis la VM
# (depuis PowerShell sur la VM)
Test-NetConnection -ComputerName 192.168.56.112 -Port 443
```

---

*Ticket : VIR-26 | Projet : Virology | Promo MSc 2027 — Epitech Nancy*
