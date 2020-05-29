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

class FlickrGalleryModule extends CacheableModule {
	
	protected $timeOut = 120;
	
	public function render($runData){
		if($runData->getAjaxMode()){
			$this->setIncludeDefaultJs(false);	
		}
		$out = parent::render($runData);
		
		return $out;	
	}
	
	public function build($runData){

		$fh = FlickrHandler::instance();
		
		// determine mode. if either photosetId or groupName given - enter specific mode.
		// if not - just search photos.
		
		$pl = $runData->getParameterList();
		
		$contentOnly = $pl->getParameterValue("contentOnly");
		
		$photosetId = $pl->getParameterValue("photosetId");
		$groupId = $pl->getParameterValue("groupId");
		$groupUrl = $pl->getParameterValue("groupUrl");
		
		$pageNumber = $pl->getParameterValue("pageNumber");
		if($pageNumber === null){
			$pageNumber = 1;
		}
		
		$perPage = $pl->getParameterValue("perPage");
		if($perPage == null || !is_numeric($perPage) || $perPage<1 || $perPage > 100){
			$perPage = 30;	
		}
		$size = $pl->getParameterValue("size");
		if($size == null || ($size != "square" && $size != "small" && $size != "thumbnail" && $size != "medium")){
			$size = "thumbnail";	
		}
		
		$limit = $pl->getParameterValue("limitPages");
		if(!is_numeric($limit) || $limit<1){
			$limit = null;	
		}

		if($photosetId != null){
			
			// get photoset info
			$photoset = $fh->photosets_getInfo($photosetId);
			if($photoset == null){
				throw new ProcessException(_("Can not fetch photoset data - photoset might not exist or not be public (or other problem)."), "no_photoset");	
			}

			// take photos from the photoset!!	
			$result = $fh->photosets_getPhotos($photosetId);
			$photos = $result['photo'];
			
			$totalPhotos = $photoset['photos'];
			
			// slice to get only page results
			$photos = array_slice($photos, ($pageNumber-1)*$perPage , $perPage);
			
			$userId = $photoset['owner'];
			for($i=0; $i<count($photos); $i++){
				$photos[$i]['src'] = $fh->buildPhotoURL($photos[$i], $size);
				$photos[$i]['href'] = 'http://www.flickr.com/photos/'.$userId.'/'.$photos[$i]['id'];	
			}
			
		}elseif($groupId !== null || $groupUrl != null){
			// take photos from a group
			if($groupId !== null){
				// get group by ID
				$group = $fh->groups_getInfo($groupId); 	
			} else {
				$group = $fh->urls_lookupGroup($groupUrl);
				$groupId = $group['id'];
			}
			if($group == null){
				throw new ProcessException(_("Can not fetch group data."), "no_group");	
			}

			$result = $fh->groups_pools_getPhotos($groupId,null, null, null, $perPage, $pageNumber);
			$totalPhotos = $result['total'];
			
			$photos = $result['photo'];
			for($i=0; $i<count($photos); $i++){
				$photos[$i]['src'] = $fh->buildPhotoURL($photos[$i], $size);
				$photos[$i]['href'] = 'http://www.flickr.com/photos/'.$photos[$i]['owner'].'/'.$photos[$i]['id'];	
			}

		}else{
		 	// just search. woooo. 	this is COOOOOL!!!
		 	
		 	$args = array(); //search parameters
		 	$userName = $pl->getParameterValue("userName");
		 	if($userName){
		 		// get user
		 		$user = $fh->people_findByUsername($userName);
		 		if($user == null){
		 			throw new ProcessException(sprintf(_('Sorry, user <em>%s</em> could not be found.'), $userName), "no_user");
		 		}
		 		
		 		$args['user_id'] = $user;	
		 	}
		 	
		 	$tags = $pl->getParameterValue("tags");
		 	if($tags){
		 		$args['tags'] = $tags;	
		 	}
		 	
		 	$tagMode = $pl->getParameterValue("tagMode");
		 	if($tagMode == "any" || $tagMode == "all"){
		 		$args['tag_mode'] = $tagMode;
		 	}
		 	
		 	$sort = $pl->getParameterValue("sort");

		 	if(count($args) == 0){
		 		$result = $fh->photos_getRecent(null, $perPage, $pageNumber);
		 		$photos = $result['photo'];
		 		$totalPhotos = $result['total'];
		 	}else{
		 	
		 		// sort does not count.
		 		if($sort && in_array($sort, array("date-posted-asc",
							"date-posted-desc",
							"date-taken-asc",
							"date-taken-desc",
							"interestingness-desc",
							"interestingness-asc",
							"relevance"))){
					$args['sort']=$sort;
				}
		 	
			 	$args['per_page'] = $perPage;
			 	$args['page'] = $pageNumber;
			 	$result = $fh->photos_search($args );
			 	$photos = $result['photo'];
			 	$totalPhotos = $result['total'];
		 	}
		 	for($i=0; $i<count($photos); $i++){
				$photos[$i]['src'] = $fh->buildPhotoURL($photos[$i], $size);
				$photos[$i]['href'] = 'http://www.flickr.com/photos/'.$photos[$i]['owner'].'/'.$photos[$i]['id'];	
			}
			
		}
		
		// build urls
		
		$pagerData = array();
		$totalPages = ceil($totalPhotos/$perPage);
		if($limit){
			$totalPages = min($limit, $totalPages);
		}
		$pagerData['total_pages'] = $totalPages;
		
		$pagerData['current_page'] = $pageNumber;
		
		$runData->contextAdd("pagerData", $pagerData);
		
		$runData->contextAdd("photos", $photos);
		$runData->contextAdd("size", $size);
		
		if($contentOnly){
			$runData->contextAdd("contentOnly", true);
		}
		
		$runData->contextAdd("makeHoverTitles", true);//(bool)$pl->getParameterValue("makeHoverTitles"));
		$runData->contextAdd("disableBrowsing", (bool)$pl->getParameterValue("disableBrowsing"));
		
		//put parameters into context
		if(!$runData->getAjaxMode()){
			$parameters = $pl->getParametersByType("MODULE");
			$runData->contextAdd("parameters", $parameters);	
		}
	}
}
