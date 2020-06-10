<?php


namespace Wikidot\Facade\Exception;

use Wikidot\Facade\Exception;



class WrongArguments extends Exception {
    protected $code = 406;
}