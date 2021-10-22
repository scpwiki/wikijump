<?php

declare(strict_types=1);

namespace Wikijump\View\Composers;

use Illuminate\Support\Facades\URL;
use Illuminate\View\View;
use Wikidot\Utils\GlobalProperties;

class BaseComposer
{
    /** Create a new mocked page composer. */
    public function __construct()
    {
    }

    /** Bind data to the view. */
    public function compose(View $view)
    {
        $canonical = URL::current();

        $view
            ->with('canonical', $canonical)
            ->with('social_url', $canonical)
            ->with('HTTP_SCHEMA', GlobalProperties::$HTTP_SCHEMA)
            ->with('URL_DOMAIN', GlobalProperties::$URL_DOMAIN)
            ->with('URL_HOST', GlobalProperties::$URL_HOST)
            ->with('SERVICE_NAME', GlobalProperties::$SERVICE_NAME);
    }
}
