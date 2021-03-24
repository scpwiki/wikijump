<?php

namespace Wikidot\Modules\Wiki\Image;


use Wikidot\Utils\CacheableModule;
use Wikidot\Utils\FlickrHandler;
use Wikidot\Utils\ProcessException;

class FlickrGalleryViewPhotoModule extends CacheableModule
{

    protected $timeOut = 120;

    public function build($runData)
    {
        $pl =  $runData->getParameterList();
        $photoId = $pl->getParameterValue("photoId");

        $flickr = FlickrHandler::instance();
        $secret = $runData->getParameterList()->getParameterValue("secret");
        $photo = $flickr->photos_getInfo($photoId, $secret);
        if ($photo == null) {
            $runData->contextAdd("nophoto", true);
            throw new ProcessException(_("The photo cannot be loaded."));
        }

        $sizes = $flickr->photos_getSizes($photoId);

        $size = "Medium"; // perhaps sometimes original??? MUST BE UPPERCASED
        $src = $flickr->buildPhotoURL($photo, $size);
        $dimensions = $sizes[$size]['_attributes'];

        $runData->contextAdd("photoSrc", $src);
        $runData->contextAdd("photoUrl", $photo['urls']['url'][0]['_value']);
        $runData->contextAdd("photo", $photo);
        $runData->contextAdd("sizes", $sizes);
        $runData->contextAdd("size", $size);

        $runData->contextAdd("dimensions", $dimensions);
    }
}
