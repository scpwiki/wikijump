<?php

declare(strict_types=1);

namespace Wikijump\View\Composers;

use Illuminate\View\View;

class FrameComposer
{
    public function __construct()
    {
    }

    /** Bind data to the view. */
    public function compose(View $view)
    {
        $view->with('header_img_url', '/files--static/media/logo-outline.min.svg');
    }
}
