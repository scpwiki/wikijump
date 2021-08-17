<?php

declare(strict_types=1);

namespace Wikijump\View\Components;

use Illuminate\View\Component;
use Illuminate\View\View;

/**
 * Layout handler for authenticated users.
 *
 * @package Wikijump\View\Components
 */
class AppLayout extends Component
{
    /**
     * Get the view / contents that represents the component.
     *
     * @return View
     */
    public function render(): View
    {
        return view('layouts.app');
    }
}
