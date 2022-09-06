<?php
declare(strict_types=1);

return [
    /**
     * The required length for a username.
     */

    'username_min' => 1,
    'username_max' => 20,

    /**
     * How many username changes a user is allowed.
     * A farm admin can force changes above this limit.
     */
    'username_change_limit' => 3,

    /**
     * Forbidden usernames.
     */
    'forbidden_usernames' => [
        '/^www[0-9]*$/',
        '/^[0-9]*www$/',
        '/^mail$/',
        '/^\-/',
        '/\-$/',
        '/^dev$/',
        '/^blog$/',
        '/^support$/',
        '/^helpdesk$/',
        '/wikidot/',
        '/wikijump/',
        '/^pro$/',
        '/^web$/',
        '/^ssl$/',
        '/^tls$/',
        '/^payment[s]?$/',
        '/^pay$/',
        '/^secure$/',
        '/^service[s]?$/',
        '/^guru$/',
        '/^admin$/',
        '/^administrator$/',
        '/^mod$/',
        '/^moderator$/',
        '/^staff$/',
        '/^anon$/',
        '/^anonymous$/',
        '/^unknown$/',
        '/^unknown (author|user|writer|person)$/',
        '/^guest/',
        '/^root$/',
        '/^error$/',
        '/^null$/',
        '/^undefined$/',
        '/^bot$/',
        '/^robot$/',
        '/^O5-\w+$/',
        '/^SCP-\w+$/',
    ],

    /**
     * Forbidden site subdomains.
     */
    'forbidden_subdomains' => [
        '/^www[0-9]*$/',
        '/^[0-9]*www$/',
        '/^www\-/',
        '/^mail$/',
        '/^ftp$/',
        '/^http[s]?$/',
        '/^\-/',
        '/\-$/',
        '/^_/',
        '/_$/',
        '/^dev$/',
        '/^staging$/',
        '/^prod$/',
        '/^production$/',
        '/^admin$/',
        '/^root$/',
        '/^error$/',
        '/^null$/',
        '/^undefined$/',
        '/^blog$/',
        '/^support$/',
        '/^helpdesk$/',
        '/wikidot/',
        '/wikijump/',
        '/^pro$/',
        '/^mail$/',
        '/^film$/',
        '/^porn/',
        '/^spam/',
        '/^web$/',
        '/^ssl$/',
        '/^tls$/',
        '/^payment[s]?$/',
        '/^pay$/',
        '/^secure$/',
        '/^service[s]?$/',
        '/^static[0-9]*/',
        '/^img$/',
        '/^image[s]?$/',
        '/^stat[s]?$/',
        '/^your\-?site$/',
        '/^template\-/',
        '/^your\-?wiki$/',
        '/^wdupload$/',
        '/^wjupload$/',
        '/^file[s]?$/',
        '/^api[0-9]*/',
        '/^svc$/',
    ],

    /** How many contacts a user may have. */
    'contact_limit' => 1000,

    /** How many simultaneous contact requests a user may have pending. */
    'contact_request_limit' => 25,

    /** The filesystem driver avatars should use. */
    'avatar_disk' => env('APP_ENV') === 'local' ? 'public' : 's3',

    /**
     * Social media links that appear in the footer of emails.
     *
     * Each array element is an array with the following keys:
     * - name: the name of the social media service (MJML syntax)
     * - url: the URL to link to
     * - text: the text to display
     * - src: icon image source
     *
     * None of the keys are required.
     */
    'mail_social_links' => [
        [
            'name' => 'web',
            'url' => 'https://www.wikijump.org',
            'text' => 'Blog',
        ],
        [
            'name' => 'twitter',
            'url' => 'https://twitter.com/getwikijump',
            'text' => 'Twitter',
        ],
        [
            'name' => 'github',
            'url' => 'https://github.com/scpwiki/wikijump',
            'text' => 'GitHub',
        ],
    ],
];
