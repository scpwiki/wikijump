<?php
/**
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 * 
 * @category Wikidot
 * @package Wikidot
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

class FlickrGalleryViewPhotoModule extends CacheableModule {
	
	protected $timeOut = 120;
	
	public function build($runData){
		$pl =  $runData->getParameterList();
		$photoId = $pl->getParameterValue("photoId");
		
		$flickr = FlickrHandler::instance();
		$secret = $runData->getParameterList()->getParameterValue("secret");
		$photo = $flickr->photos_getInfo($photoId, $secret);
		if($photo == null){
			$runData->contextAdd("nophoto", true);
			throw new ProcessException(_("The photo can not be loaded."));
		}
		
		$sizes = $flickr->photos_getSizes($photoId);

		$size = "Medium"; // perhaps sometimes original??? MUST BE UPPERCASED
		$src = $flickr->buildPhotoURL($photo, $size);
		$dimensions = $sizes[$size]['_attributes'];

		$runData->contextAdd("photoSrc", $src);
		$runData->contextAdd("photoUrl", $photo['urls']['url'][0]['_value']);
		$runData->contextAdd("photo", $photo);
		$runData->contextAdd("sizes", $sizes);
		$runData->contextAdd("size",$size);
		
		$runData->contextAdd("dimensions", $dimensions);

	}	
}
