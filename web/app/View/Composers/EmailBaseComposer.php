<?php

declare(strict_types=1);

namespace Wikijump\View\Composers;

use Illuminate\View\View;
use Wikidot\Utils\GlobalProperties;

const EMAIL_LOGO_PATH = '/files--static/media/logo.png';

class EmailBaseComposer
{
    protected string $logo_src;

    public function __construct()
    {
        // e.g. https://www.wikijump.dev/files--static/media/logo.png
        $this->logo_src =
            GlobalProperties::$HTTP_SCHEMA .
            '://' .
            GlobalProperties::$URL_DOMAIN .
            EMAIL_LOGO_PATH;
    }

    /** Bind data to the view. */
    public function compose(View $view)
    {
        $view
            ->with('HTTP_SCHEMA', GlobalProperties::$HTTP_SCHEMA)
            ->with('URL_DOMAIN', GlobalProperties::$URL_DOMAIN)
            ->with('URL_HOST', GlobalProperties::$URL_HOST)
            ->with('SERVICE_NAME', GlobalProperties::$SERVICE_NAME)
            ->with('logo_src', $this->logo_src);
    }
}
