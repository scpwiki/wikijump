<?php

declare(strict_types=1);

namespace Wikijump\View\Composers;

use Illuminate\View\View;
use Wikidot\Utils\GlobalProperties;

class EmailBaseComposer
{
    public function __construct()
    {
    }

    /** Bind data to the view. */
    public function compose(View $view)
    {
        $view
            ->with('HTTP_SCHEMA', GlobalProperties::$HTTP_SCHEMA)
            ->with('URL_DOMAIN', GlobalProperties::$URL_DOMAIN)
            ->with('URL_HOST', GlobalProperties::$URL_HOST)
            ->with('SERVICE_NAME', GlobalProperties::$SERVICE_NAME)
            ->with('social_links', config('wikijump.mail_social_links'));
    }
}
