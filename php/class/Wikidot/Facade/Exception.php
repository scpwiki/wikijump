<?php


namespace Wikidot\Facade;

use Exception as Exception;



abstract class Exception extends Exception {
	protected $code = 0; // each subclass should define its error code (int)
}