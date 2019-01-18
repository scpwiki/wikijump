<?php

abstract class Wikidot_Facade_Exception extends Exception {
	protected $code = 0; // each subclass should define its error code (int)
}