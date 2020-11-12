<?php




/**
 * PHP error exception.
 *
 */
class PHPErrorException extends Exception {

    private $context = null;

    public function __construct($code, $message, $file, $line, $context = null) {
        parent::__construct($message, $code);
        $this->file = $file;
        $this->line = $line;
        $this->context = $context;
    }
}
