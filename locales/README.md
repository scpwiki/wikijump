# Wikijump Translations

This folder contains the localization files needed by the various Wikijump projects.

> ### IMPORTANT:
>
> The configuration file format to be used for locales has not been decided yet. It will be either YAML or TOML. For right now, YAML is being used.
>
> Despite this, it is safe to make translations, as it is relatively simple to convert between YAML and TOML and any work done won't be wasted.

### Relevant Documentation:

- [ICU Syntax](https://formatjs.io/docs/core-concepts/icu-syntax/)

### File Naming

Locale file names have a specific format. A locale will start with the basic language tag, e.g. `en` for English. Then, optionally, it may be followed by a region code, e.g. `-us` (`en-us`). You might have to look up what these language codes and region identifiers are, as they're often not what you expect.

Locale files with a region code will _inherit any missing translations from the base language_. That means if a translation string doesn't exist for `en-us`, the `en` locale file will be searched next. What that means is that you _do not need to copy the entire file for region-specific locales_. Instead, just change the translations that you need to, and leave the rest alone. Using English as an example, the `en-us` file would handle using the word "color" different from `en-gb`, which would use "colour" instead.

### Key Names

For the sake of consistency, the following rules are followed when naming keys:

- Namespaces, objects, groups, categories, etc, whatever you may call them, are _lowercased_.
- Otherwise, key names are _UPPERCASED_.
- Words are separated with _snake_case_.

That will look like this:

```yaml
namespace:
  KEY_NAME: Translation string!

other_namespace:
  deeply_nested:
    OTHER_KEY_NAME: Another translation string!
```

If you're a contributor who came to this repository to help with translations, you may be unfamiliar with how these configuration files actually get used. In general, it looks something like this:

```yaml
login:
  buttons:
    FORGOT_PASSWORD: Forgot your password? Click here.
```

```svelte
<!-- Starts the "forgot password" process -->
<button>
  <span>{$t("login.buttons.FORGOT_PASSWORD")}</span>
</button>
```
