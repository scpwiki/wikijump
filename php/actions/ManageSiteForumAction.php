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

class ManageSiteForumAction {
	
	public function isAllowed($runData){
		WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));	
		return true;
	}
	
	public function perform($r){}
	
	public function activateForumEvent($runData){
		$site = $runData->getTemp("site");
		
		$db = Database::connection();
		$db->begin();
		
		// copy forum settings from template
		$c = new Criteria();
		$c->add("unix_name", "template-".$site->getLanguage());
		$templateSite = DB_SitePeer::instance()->selectOne($c);

		$fs = $templateSite->getForumSettings();
		$fs->setNew(true);
		$fs->setSiteId($site->getSiteId());
		$fs->save();
		
		// create extra categories? no.
		
		// copy pages
		$d = new Duplicator();
		$d->setOwner($runData->getUser());
		
		// copy "forum" category
		$fc = DB_CategoryPeer::instance()->selectByName("forum", $templateSite->getSiteId());
		$d->duplicateCategory($fc, $site);
		
		// recompile category.
		$od = new Outdater();
		$od->recompileCategory(DB_CategoryPeer::instance()->selectByName("forum", $site->getSiteId()));
		
		// create a "Hidden" forum group and "Deleted" category
		
		$group = new DB_ForumGroup();
		$group->setSiteId($site->getSiteId());
		$group->setName("Hidden");
		$group->setVisible(false);
		$group->save();
		
		$del = new DB_ForumCategory();
		$del->setSiteId($site->getSiteId());
		$del->setName(_("Deleted threads"));
		$del->setDescription(_("Deleted forum discussions should go here."));
		$del->setPermissions("t:;p:;e:;s:");
		$del->setGroupId($group->getGroupId());
		$del->save();
		
		$category = new DB_ForumCategory();
		$category->setName(_("Per page discussions"));
		$category->setDescription(_("This category groups discussions related to particular pages within this site."));
		$category->setPerPageDiscussion(true);
		$category->setSiteId($site->getSiteId());
		$category->setGroupId($group->getGroupId());
		$category->save();
		
		$db->commit();
		if (GlobalProperties::$UI_SLEEP) { sleep(1); }
	}

	public function saveForumLayoutEvent($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		
		$json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
		
		$cats0 = $json->decode($pl->getParameterValue("categories"));
		$groups0 = $json->decode($pl->getParameterValue("groups"));

		$db = Database::connection();
		$db->begin();
		
		// compare against stored groups and categories. add if necessary, delete if necessary etc.
		for($i = 0; $i < count($groups0); $i++){
			$group = $groups0[$i];
			$g = null;
			if($group['group_id'] == null){
				// new group, add to database!
				$g = new DB_ForumGroup();
				$g->setName(trim($group['name']));
				$g->setDescription(trim($group['description']));
				$g->setVisible($group['visible']);
				$g->setSiteId($site->getSiteId());
				$g->setSortIndex($i);
				$g->save();	
			} else {
				$c = new Criteria();
				$c->add("site_id", $site->getSiteId());
				$c->add("group_id", $group['group_id']);
				$g = DB_ForumGroupPeer::instance()->selectOne($c);
				if($g == null){
					throw new ProcessException(_("Error fatching one of the forum groups."));
				}
				// update values
				$changed = false;
				if($g->getName() !== trim($group['name'])){
					$g->setName(trim($group['name']));
					$changed = true;	
				}
				if($g->getDescription() !== trim($group['description'])){
					$g->setDescription(trim($group['description']));
					$changed = true;
				}
				if($g->getVisible() !== $group['visible']){
					$g->setVisible($group['visible']);
					$changed = true;
				}
				if($g->getSortIndex() !== $i){
					$g->setSortIndex($i);
					$changed = true;
				}
				
				if($changed){
					$g->save();
				}
					
			}
			// now proceed with categories for this group!!!
			$cates = $cats0[$i];
			for($j=0; $j<count($cates); $j++){
				$cat = $cates[$j];
				if($cat['category_id'] == null){
					// new category!
					$ca = new DB_ForumCategory();
					$ca->setName(trim($cat['name']));
					$ca->setDescription(trim($cat['description']));
					$ca->setMaxNestLevel($cat['max_nest_level']);
					$ca->setSiteId($site->getSiteId());
					$ca->setGroupId($g->getGroupId());
					$ca->setSortIndex($j);
			
					$ca->save();
				}else{
					$c = new Criteria();
					$c->add("site_id", $site->getSiteId());
					$c->add("category_id", $cat['category_id']);
					$ca = DB_ForumCategoryPeer::instance()->selectOne($c);
					if($ca == null){
						throw new ProcessException(_("Error fatching one of the forum categories."));
					}
					$changed = false;
					if($ca->getName() !== trim($cat['name'])){
						$ca->setName(trim($cat['name']));
						$changed = true;	
					}
					if($ca->getDescription() !== trim($cat['description'])){
						$ca->setDescription(trim($cat['description']));
						$changed = true;
					}
					if($ca->getMaxNestLevel() !== $cat['max_nest_level']){
						$ca->setMaxNestLevel($cat['max_nest_level']);
						$changed = true;
					}
					if($ca->getSortIndex() !== $j){
						$ca->setSortIndex($j);
						$changed = true;
					}
					if($ca->getGroupId() != $g->getGroupId()){
						$ca->setGroupId($g->getGroupId());	
						$changed = true;
					}
					
					if($changed){
						$ca->save();	
					}
					
				}
			}
				
		}
		
		// and deleted categories
		
		$dcats = $json->decode($pl->getParameterValue("deleted_categories"));
		foreach($dcats as $dcat){
			$c = new Criteria();
			$c->add("site_id", $site->getSiteId());
			$c->add("category_id", $dcat);
			
			// check if empty
			$cacount = DB_ForumThreadPeer::instance()->selectCount($c);
			if($cacount>0){
					throw new ProcessException(_("One of the categories marked for deletation was not empty."));	
			}
			DB_ForumCategoryPeer::instance()->delete($c);
		}
		
		// now process deleted groups...
		
		$dgroups = $json->decode($pl->getParameterValue("deleted_groups"));
		for($i = 0; $i < count($dgroups); $i++){
			$group = $dgroups[$i];
			// check if has group_id - if not, this should not be in the database...
			if($group['group_id'] !== null){
				$c = new Criteria();
				$c->add("site_id", $site->getSiteId());
				$c->add("group_id", $group['group_id']);
				
				$cacount = DB_ForumCategoryPeer::instance()->selectCount($c);
				if($cacount>0){
					throw new ProcessException(_("One of the groups marked for deletation was not empty."));	
				}
				
				DB_ForumGroupPeer::instance()->delete($c);
			}	
		}
		
		// and deleted categories
		
		$dcats = $json->decode($pl->getParameterValue("deleted_categories"));
		foreach($dcats as $dcat){
			$c = new Criteria();
			$c->add("site_id", $site->getSiteId());
			$c->add("category_id", $dcat);
			
			// check if empty
			$cacount = DB_ForumThreadPeer::instance()->selectCount($c);
			if($cacount>0){
					throw new ProcessException(_("One of the categories marked for deletation was not empty."));	
			}
			DB_ForumCategoryPeer::instance()->delete($c);
		}
		
		$outdater = new Outdater();
		$outdater->forumEvent("outdate_forum");
		
		$db->commit();
		if (GlobalProperties::$UI_SLEEP) { sleep(1); }
			
	}
	
	public function saveForumDefaultNestingEvent($runData){
		$pl = $runData->getParameterList();
		$site = $runData->getTemp("site");
		
		$db = Database::connection();
		$db->begin();
		
		$settings = $site->getForumSettings();
		$level = $pl->getParameterValue("max_nest_level");
		if($level === null || !is_numeric($level)|| $level<0 || $level >10){
			throw new ProcessException(_("Level value invalid."));	
		} 
		$settings->setMaxNestLevel($level);
		
		$settings->save();
		$outdater = new Outdater();
		$outdater->forumEvent("outdate_forum");
		$db->commit();
		if (GlobalProperties::$UI_SLEEP) { sleep(1); }
	}
	
	public function saveForumPermissionsEvent($runData){
		$pl =  $runData->getParameterList();
		$site = $runData->getTemp("site");
		$siteId = $site->getSiteId();
		$json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
		$cats = $json->decode($pl->getParameterValue("categories"));
		
		$db = Database::connection();
		
		$db->begin();
		/* for each category 
		 *  - get a category from database
		 *  - check if permissions has changed
		 *  - if changed: update
		 */
		 
		 foreach($cats as $cat){
		 	$categoryId = $cat['category_id'];
		 	$c = new Criteria();
			$c->add("site_id", $site->getSiteId());
			$c->add("category_id", $cat['category_id']);
			$ca = DB_ForumCategoryPeer::instance()->selectOne($c);
			
			if($cat == null){
				throw new ProcessException("Invalid category.");
			}
			//validate permstring
			$permstring = $cat['permissions'];
			$p2 = explode(";", $permstring);
			foreach($p2 as $perm){
				if($permstring && preg_match("/^[tpes]:[armo]{0,4}$/", $perm) == 0){
					throw new ProcessException(_("Error saving permissions - invalid internal format. Please try again and contact admins if the problem repeats."));
				}	
			}
			if($ca->getPermissions() !== $cat['permissions']){
				$ca->setPermissions($cat['permissions']);
				$ca->save();	
			}

		 }
		
		$defaultPermissions = $pl->getParameterValue("default_permissions");
		$p2 = explode(";", $defaultPermissions);
		foreach($p2 as $perm){
			if(preg_match("/^[tpes]:[armo]{0,4}$/", $perm) == 0){
				throw new ProcessException(_("Error saving permissions - invalid internal format. Please try again and contact admins if the problem repeats."));
			}
		}
		$fSettings = $site->getForumSettings();
		
		if($fSettings->getPermissions() !== $defaultPermissions){
			$fSettings->setPermissions($defaultPermissions);
			$fSettings->save();	
		}

		$db->commit();
		if (GlobalProperties::$UI_SLEEP) { sleep(1); }
	}
	
	public function savePerPageDiscussionEvent($runData){
		$pl =  $runData->getParameterList();
		$site = $runData->getTemp("site");
		$siteId = $site->getSiteId();
		$json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
		$cats0 = $json->decode($pl->getParameterValue("categories"));
		
		$db = Database::connection();
		$db->begin();
		
		$outdater = new Outdater();
		foreach($cats0 as $category){
			$categoryId	= $category['category_id'];
			$c = new Criteria();
			$c->add("category_id", $categoryId);
			$c->add("site_id", $siteId); // for sure ;-)
			$dCategory = DB_CategoryPeer::instance()->selectOne($c);
			
			// now compare
			$changed = false;
			if($category['per_page_discussion'] !== $dCategory->getPerPageDiscussion()){
				$dCategory->setPerPageDiscussion($category['per_page_discussion']);
				$changed = true;	
			}
			if($changed){
				$dCategory->save();	
				// outdate category too
				$outdater->categoryEvent("category_save", $dCategory);
			}
		}

		$outdater->forumEvent("outdate_forum");
		
		$db->commit();
		
		if (GlobalProperties::$UI_SLEEP) { sleep(1); }
	}
	
}
