<?php

return [
    /**
     * User defaults
     */
    'user' => [
        /**
         * These are legacy Wikidot defaults from the OzoneUserSettings class.
         * They should be evaluated and modified or removed as needed.
         */
        'receive_invitations' => true,
        'receive_pm' => 'a',
        'notify_online' => '*',
        'notify_feed' => '*',
        'notify_email' => null,
        'receive_newsletter' => true,
        'receive_digest' => true,
        'allow_site_newsletters_default' => true,
        'max_sites_admin' => 3, # This should be controlled by RBAC, not a setting.
        /**
         * Everything below are currently examples and not in use.
         */

        /**
         * Whether to allow PMs from other users. Farm admins are exempt.
         */
        'allow_pms' => true,

        /**
         * Whether the user is banned from the farm.
         */
        'banned' => false,

        /**
         * Whether to show the "last online at" field for the user.
         */
        'show_last_online_time' => true,

        /**
         * Whether a user can be found in a search by real name.
         */
        'allow_search_by_real_name' => false,

        /**
         * Per-site settings are what they sound like, a user may want one site
         * to behave differently than another.
         */
        'per_site_settings' => [
            /**
             * Whether to show or hide the vote module on pages for this site.
             */
            'show_vote_module' => true,
        ],
    ],
];
