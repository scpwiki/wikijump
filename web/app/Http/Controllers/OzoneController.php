<?php

namespace Wikijump\Http\Controllers;

class OzoneController extends Controller
{
    public function handle($path = "")
    {
        ob_start();
        require base_path('web/index.php');
        return response(ob_get_clean());
    }
}
