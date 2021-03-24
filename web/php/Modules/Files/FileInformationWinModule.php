<?php

namespace Wikidot\Modules\Files;


use Wikidot\DB\FilePeer;

use Ozone\Framework\SmartyModule;

class FileInformationWinModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $fileId = $pl->getParameterValue("file_id");

        $file = FilePeer::instance()->selectByPrimaryKey($fileId);

        if ($file == null || $file->getSiteId() != $runData->getTemp("site")->getSiteId()) {
            $runData->ajaxResponseAdd("status", "wrong_file");
            $runData->ajaxResponseAdd("message", _("Error getting file information."));
            $runData->setModuleTemplate("Empty");
            return;
        }

        $runData->contextAdd("file", $file);
    }
}
