# Components

This package contains [Svelte](https://svelte.dev/) components that serve
the websites in the Wikijump frontend. These add reactivity to the page to
facilitate user interaction.

## Stub replacement

Because the primary experience of using the site for most users is simply
reading without interaction, the reactivity of the frontend is designed to
at least be readable with JavaScript disabled. This helps reduce unnecessary
load on slower browsers, while also ensuring that the Wikijump frontend is
not entirely reliant on JavaScript to operate. At the bare minimum, the
entire content of a Wikijump article should be readable without JavaScript.

However, there are some parts of the page that are necessary for content
presentation that require reactivity to work properly. An example of this is
the tabview &mdash; it is necessary for the user to interact with the
component to choose which tab they wish to read, however, JavaScript is
needed for this interaction, without which a user will be unable to read
most of the tabs.

Where this situation occurs, the HTML returned by the server contains a
simplistic stub component that contains all the necessary information for
content presentation in a form where it is all readable at the same time.
This ensures that a user without JavaScript is able to experience all the
content of the article. For users with JavaScript, a full Svelte component
will replace the stub component, adding the reactivity that is needed to
interact with it.

Each component that works in this way has a certain syntax that the HTML
returned by the server must follow in order for the content to be
propagated into the Svelte component. These requirements are documented in
these components, and the Blade templates that construct the stub components
must be sure to follow suit.

- TODO: How are components that replace stub components designated?
