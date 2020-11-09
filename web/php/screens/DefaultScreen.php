<?php
class DefaultScreen extends SmartyScreen
{

    public function build($runData)
    {
        $runData->contextAdd("testkey", "testval");
    }
}
