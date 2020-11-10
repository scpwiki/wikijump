<?php
class ProcessException extends Exception
{

    protected $status;

    public function __construct($message, $status = "not_ok")
    {
       // some code

       // make sure everything is assigned properly
        parent::__construct($message, 1);
        $this->status = $status;
    }

    public function getStatus()
    {
        return $this->status;
    }
}
