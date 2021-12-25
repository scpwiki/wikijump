# Wikijump Translations

This folder contains the localization files needed by the various Wikijump projects.

### Relevant Documentation:

- [Fluent Syntax](https://projectfluent.org/fluent/guide/)

### Background

Wikijump universally uses [Project Fluent](https://projectfluent.org/) for localization. This is the same system Firefox uses. All Fluent (`.ftl`) files can be found in the `fluent` folder.

There is a singular exception to this, which is the `cmftml` folder. More information can be found in [that folder's `README`](https://github.com/scpwiki/wikijump/tree/develop/locales/cmftml)..

### How this works

The `fluent` folder contains many subfolders. Each folder represents a "component", or chunk of related translation strings. For example, the `notification-bell` component holds strings related to the small bell that informs the user when they have new notifications.

Components have `.ftl` files named after a locale, so for example a component may have a `en.ftl` file, a `de.ftl` file, and so on. To add translations for a locale, you simply need to add a new `.ftl` file with that locale's language tag. You may need to look up what these language codes are, as they're often not what you expect.

When adding new translations, you need to use the `en.ftl` file as your basis, or else the message keys won't match. It's recommended you copy the `en.ftl` file, rename it, and then change the strings to match the locale.

You don't need to do anything but add a new `.ftl` file when translating. The backend and frontend automatically figure out what they need to do from the file structure of the `fluent` folder.

### How this gets used

If you're a contributor who came to this repository to help with translations, you may be unfamiliar with how these files actually get used. In general, it looks something like this:

```ftl
forgot-password = Forgot Password
  .question = Forgot your password?
```

```svelte
<!-- Starts the "forgot password" process -->
<button>
  <span>{$t("forgot-password.question")}</span>
</button>
```
