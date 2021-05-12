<?php
declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Illuminate\Contracts\Foundation\Application;
use Illuminate\Contracts\Routing\ResponseFactory;
use Illuminate\Http\Response;

class OzoneController extends Controller
{
    /**
     * We use the OzoneController as the route of last resort.
     * $path is taken by the require() statement to route correctly within Ozone.
     * @param string $path
     * @return Application|ResponseFactory|Response
     * @noinspection PhpUnusedParameterInspection
     */
    public function handle($path = '')
    {
        ob_start();
        require base_path('web/ozone.php');
        return response(ob_get_clean());
    }
}
