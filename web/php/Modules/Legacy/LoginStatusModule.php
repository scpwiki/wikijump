<?php

declare(strict_types=1);

namespace Wikidot\Modules\Legacy;
use Ozone\Framework\Module;

/**
 * Module to display login status at top bar.
 * @package Wikidot\Modules
 */
class LoginStatusModule extends Module
{
    public function render($runData)
    {
        /**
         * The reason we're doing it this way instead of injecting the view
         * directly is that Smarty will cache the view, but not a rendered
         * module like this. As we move to blade both will be axed.
         */
        return view('legacy.loginstatus');
    }
}
