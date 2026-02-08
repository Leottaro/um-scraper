# UMscraper

UMscraper est un programme Rust qui récupère vos notes du dernier semestre de la section `Mon dossier > Notes & Résultats` du portail ENT de l'Université de Montpellier et vous notifie par email en cas de changement.

## Setup

### Prérequis

-   [Rust](https://rust-lang.org/tools/install/)
-   [Geckodriver](https://github.com/mozilla/geckodriver#installation)
-   Un compte Gmail (désolé Alban)

### Paramètres:

le fichier `umscraper.yaml` à la racine du projet doit contenir :

```yaml
ent_login_email: '[...]@etu.umontpellier.fr'
ent_password: '...'
gmail_login_email: '[...]@gmail.[...]'
gmail_login_password: app password
gmail_gmail_from_email: different mail (alias) or remove this part
to_emails:
- mail1@gmail.com
- mail2@hotmail.com
- ...
data_file: ./notes.yaml
sleep_time: 10m
geckodriver_port: 4444
```

### Remplir les paramètres

1. **ent_login_email**: Votre adresse email de l'Université de Montpellier.
2. **ent_password**: Votre mot de passe de l'Université de Montpellier.

3. **gmail_login_email**: L'adresse email du compte Gmail qui enverra les notifications.
4. **gmail_login_password**: Le mot de passe de application Gmail.
    - Note : Vous devez générer un mot de passe d'application depuis les paramètres de votre compte Google. Suivez [ce guide](https://support.google.com/accounts/answer/185833?hl=fr) pour générer un mot de passe d'application.
5. **gmail_from_email**: **— OPTIONNEL —** L'adresse email qui apparaîtra comme expéditeur des notifications.
    - Si non spécifié, `gmail_login_email` sera utilisé.
6. **to_emails**: Les adresses email qui recevront les notifications.

7. **sleep_time**: La durée entre chaque tentative de scraping. Vous pouvez spécifier la durée en secondes (s), minutes (m), ou heures (h).
    - Exemple : `sleep_time: "1h"` pour une heure, `sleep_time: "30m"` pour 30 minutes, ou `sleep_time: "45s"` pour 45 secondes.
8. **geckodriver_port**: Le port sur lequel sera executé geckodriver.

### Exécution de l'application

Vous pouvez simplement lancer le projet avec `cargo run`.
L'application va scraper les notes toute seule et vous enverra un mail si un changement est détecté.

### Dépannage
-   **Problèmes de connexion** : Assurez-vous que vos identifiants ENT sont corrects et que vous pouvez vous connecter manuellement.
-   **Problèmes avec Geckodriver** : Vérifiez que `geckodriver` est bien dans votre PATH. (`which geckodriver`)
-   **Problèmes d'envoi d'email** : Assurez-vous que vous avez généré un mot de passe d'application pour votre compte Gmail et que les paramètres de votre compte permettent l'envoi d'emails via SMTP.