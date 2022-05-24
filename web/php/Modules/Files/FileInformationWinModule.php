<?php

namespace Wikidot\Modules\Files;

use Ozone\Framework\SmartyModule;
use Wikijump\Services\Deepwell\Models\File;

class FileInformationWinModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");
        $fileId = $pl->getParameterValue("file_id");

        $file = File::findId($fileId);
        if ($file === null || $file->getSiteId() !== $site->getSiteId()) {
            $runData->ajaxResponseAdd("status", "wrong_file");
            $runData->ajaxResponseAdd("message", _("Error getting file information."));
            $runData->setModuleTemplate("Empty");
            return;
        }

        $runData->contextAdd("file", $file);
    }
}
