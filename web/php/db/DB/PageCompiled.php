<?php
namespace DB;

use ODate;

/**
 * Object Model class.
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
