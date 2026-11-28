# Gérer mon école pour Windows et Mac

L'application de bureau de Gérer mon école. C'est une fenêtre qui ouvre le site de Gérer mon école : les mises à jour
du site arrivent donc toutes seules dans l'application, sans la réinstaller.

Faite avec [Tauri](https://tauri.app) : le fichier d'installation pèse quelques Mo.

## Ce que fait l'application

- Écran de démarrage, et « Pas de connexion internet » avec un bouton Réessayer.
- Les liens vers d'autres sites (WhatsApp, e-mail, téléphone…) s'ouvrent dans le navigateur.
- Les documents du site (factures…) s'ouvrent dans une fenêtre à part, toujours connectée.
- Les fichiers téléchargés vont dans « Téléchargements » et s'ouvrent.
- La taille et la position de la fenêtre sont retenues ; un deuxième lancement ramène la fenêtre ouverte.

## Changer l'adresse du site (nom de domaine)

Remplacer `www.gerermonecole.com` dans ces trois fichiers :
`src-tauri/src/main.rs`, `src/index.html` et `src-tauri/capabilities/default.json`.
Puis publier une nouvelle version.

## Publier une nouvelle version

1. Augmenter `version` dans `src-tauri/tauri.conf.json` et `package.json` (par exemple 1.0.1).
2. Enregistrer, puis :

```bash
git tag v1.0.1 && git push origin main v1.0.1
```

GitHub fabrique alors `GererMonEcole-Windows.exe` et `GererMonEcole-Mac.dmg` (environ 15 minutes) et les met
dans « Releases ». Le site les propose toujours par ces liens :

- https://github.com/keydeath03/ecole-plus-desktop/releases/latest/download/GererMonEcole-Windows.exe
- https://github.com/keydeath03/ecole-plus-desktop/releases/latest/download/GererMonEcole-Mac.dmg

## Signature

Sans certificat, Windows affiche « Windows a protégé votre ordinateur » (cliquer « Informations
complémentaires » puis « Exécuter quand même ») et le Mac refuse d'ouvrir l'application la première fois
(clic droit sur Gérer mon école > Ouvrir, ou Réglages Système > Confidentialité et sécurité > Ouvrir quand même).
Une fois la société créée : compte développeur Apple (signature et notarisation) et certificat Windows.

## Travailler sur l'application

```bash
npm install
npm run dev     # lance l'application
npm run build   # fabrique l'installateur de cet ordinateur
```
