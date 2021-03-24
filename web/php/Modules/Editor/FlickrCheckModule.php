<?php

namespace Wikidot\Modules\Editor;


use Ozone\Framework\SmartyModule;
use Wikidot\Utils\FlickrHandler;

class FlickrCheckModule extends SmartyModule
{

    public function build($runData)
    {
        $flickrId = $runData->getParameterList()->getParameterValue("flickr_id");
        $secret = $runData->getParameterList()->getParameterValue("secret");
        $flickr = FlickrHandler::instance();

        $photo = $flickr->photos_getInfo($flickrId, $secret);
        if ($photo == null) {
            $runData->contextAdd("nophoto", true);
            return;
        }
        $src = $flickr->buildPhotoURL($photo, "thumbnail");
        $runData->contextAdd("flickrId", $flickrId.$secret);
        $runData->contextAdd("photoSrc", $src);
        $runData->contextAdd("photo", $photo);
    }
}
