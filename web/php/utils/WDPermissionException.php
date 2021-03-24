<?php

namespace Wikidot\Utils;

use Exception;

class WDPermissionException extends Exception
{
    protected $code = 403;
}
