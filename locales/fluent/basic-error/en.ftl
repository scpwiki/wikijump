### Basic Error HTML templates

basic-error-site-slug = <h1>No { -service-name } site exists with this address</h1>
    <p>
      The site <a href="https://{ $slug }.{ $main_domain }/"><code>{ $slug }.{ $main_domain }</code></a> does not exist.
    </p>

    <p>
      Return to <a href="https://{ $main_domain }/">{ -service-name }</a>.
    </p>

    .title = No such site - { -service-name }

basic-error-site-custom = <h1>No { -service-name } site exists with this address</h1>
    <p>
      No site has the custom domain <a href="https://{ $custom_domain }/"><code>{ $custom_domain }</code></a>.
    </p>

    <p>
      Return to <a href="https://{ $main_domain }/">{ -service-name }</a>.
    </p>

    .title = No such site - { -service-name }

basic-error-page-slug = <h1>This page does not exist</h1>
    <p>
      The page <a href="https://{ $domain }/{ $page_slug }"><code>{ $domain }/{ $page_slug }</code></a> does not exist.
    </p>

    <p>
      Return to the <a href="https://{ $domain }/">site's main page</a>.
    </p>

    .title = No such page - { $domain }

basic-error-page-fetch = <h1>Unable to fetch page data</h1>
    <p>
      Server error: Page data from <a href="https://{ $domain }/{ $page_slug }"><code>{ $domain }/{ $page_slug }</code></a> could not be loaded.
    </p>

    <p>
      Return to the <a href="https://{ $domain }/{ $page_slug }">page</a>, or the <a href="https://{ $domain }/">site's main page</a>.
    </p>

    .title = Server Error - { $domain }

basic-error-file-name = <h1>This file does not exist</h1>
    <p>
      The file <code>{ $filename }</code> on the page <code>{ $domain }/{ $page_slug }</code> does not exist.
    </p>

    <p>
      Return to the <a href="https://{ $domain }/{ $page_slug }">page</a>, or the <a href="https://{ $domain }/">site's main page</a>.
    </p>

    .title = No such page - { $domain }

basic-error-file-fetch = <h1>Unable to fetch file data</h1>
    <p>
      Server error: File data from <code>{ $filename }</code> on the page <code>{ $domain }/{ $page_slug }</code> could not be loaded.
    </p>

    <p>
      Return to the <a href="https://{ $domain }/{ $page_slug }">page</a>, or the <a href="https://{ $domain }/">site's main page</a>.
    </p>

    .title = Server Error - { $domain }

basic-error-text-block = <h1>Invalid hosted text block</h1>
    <p>
      { $reason ->
        [missing] No { $type ->
          [code] code
          [html] HTML
          *[error] text
        } block with index <code>{ $index }</code> exists.
        [invalid] The given index <code>{ $index }</code> is invalid.
        [fetch] This text block data could not be loaded.
        *[error] Unknown basic error reason: { $reason }
      }
    </p>

    <p>
      Return to the <a href="https://{ $domain }/">site's main page</a>.
    </p>

    .title = Text Block Error - { $domain }

basic-error-file-root = <h1>Invalid route</h1>
    <p>
      { -service-name } serves user-generated data from <code>{ $files_domain }</code>, but this is not a valid URL.
    </p>

    <p>
      Return to <a href="https://{ $main_domain }/">{ -service-name }</a>.
    </p>

    .title = { -service-name }

basic-error-blob-fetch = <h1>Unable to fetch blob data</h1>
    <p>
      Server error: Blob with S3 hash <code>{ $s3_hash }</code> could not be loaded.
    </p>

    .title = Server Error

basic-error-user-fetch = <h1>Unable to fetch user data</h1>
    <p>
      Server error: User with id <code>{ $user_id }</code> could not be loaded.
    </p>

    .title = Server Error

basic-error-user-avatar = <h1>Unable to fetch user avatar</h1>
    <p>
      Server error: User avatar with user id <code>{ $user_id }</code> could not be loaded.
    </p>

    .title = Server Error
