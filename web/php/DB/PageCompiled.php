<?php

namespace Wikidot\DB;


use Ozone\Framework\ODate;

/**
 * Object Model Class.
 *
 */
class PageCompiled extends PageCompiledBase
{

    public function outdate()
    {
        $dateback = new ODate();
        $dateback->addSeconds(-1000000);
        $this->setDateCompiled($dateback);
    }
}
