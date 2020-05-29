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

class AccountProfileAction extends SmartyAction {

    public function isAllowed($runData) {
        $userId = $runData->getUserId();
        if ($userId == null || $userId < 1) {
            throw new WDPermissionException(_("Not allowed. You should login first."));
        }
        return true;
    }

    public function perform($r) {}

    public function uploadAvatarEvent($runData) {
        $status = "ok"; // status variable that will be passed to template

        $pl = $runData->getParameterList();

        $file = $_FILES['userfile'];
        
        if ($file['size'] == 0) {
            $status = "zero_size";
            $runData->contextAdd("status", $status);
            return;
        }
        
        if ($file['error'] != 0) {
            $status = "other error";
            $runData->contextAdd("status", $file['error']);
            return;
        }
        
        if (!is_uploaded_file($file['tmp_name'])) {
            $status = "invalid_file";
            $runData->contextAdd("status", $status);
            return;
        }

        $fmime = FileMime::mime($file['tmp_name']);
        
        if ($fmime != "image/png" && $fmime != "image/jpeg" && $fmime != "image/gif") {
            $status = "wrong_mime";
            $runData->contextAdd("status", $status);
            $runData->contextAdd("mime", $fmime);
            return;
        }
        
        $size = getimagesize($file['tmp_name']);
        if ($size == false) {
            $status = "not_image";
            $runData->contextAdd("status", $status);
            return;
        }
        
        if ($size[0] < 16 || $size[1] < 16) {
            $status = "too_small";
            $runData->contextAdd("status", $status);
            return;
        }
        
        // new temporary files for 48 and 16 images
        $dir = WIKIDOT_ROOT . '/web/files--common/tmp/avatars-upload';
        
        $im48fn = tempnam($dir, "av") . ".png";
        $im16fn = tempnam($dir, "av") . ".png";
        
        if ($size[0] != 100 && $size[1] != 100) {
            // need to resize...	
            $w = $size[0];
            $h = $size[1];
            $r = $h / $w;
            $cmd = "convert -resize '100x100>' " . escapeshellarg($file['tmp_name']) . " " . escapeshellarg($im48fn);
            
            exec($cmd, $out);
            
            $runData->contextAdd("originalSize", $size);
            $runData->contextAdd("resized", true);
        } else {
            $cmd = "convert  " . escapeshellarg($file['tmp_name']) . " " . escapeshellarg($im48fn);
            exec($cmd);
        }
        $cmd = "convert -resize 16x16! -unsharp 0x1.0+1.0+0.10 " . escapeshellarg($im48fn) . " " . escapeshellarg($im16fn);
        exec($cmd);
        
        $runData->contextAdd("im48", basename($im48fn));
        $runData->contextAdd("im16", basename($im16fn));
        
        $runData->contextAdd("status", $status);
    
    }

    public function setAvatarEvent($runData) {
        
        $userId = $runData->getUserId();
        
        $pl = $runData->getParameterList();
        $im48 = $pl->getParameterValue("im48");
        $im16 = $pl->getParameterValue("im16");
        
        $avatarDir = WIKIDOT_ROOT . '/web/files--common/images/avatars/';
        $avatarDir .= '' . floor($userId / 1000) . '/' . $userId;
        
        mkdirfull($avatarDir);
        $tmpDir = WIKIDOT_ROOT . '/web/files--common/tmp/avatars-upload';
        rename($tmpDir . '/' . $im48, $avatarDir . '/a48.png');
        rename($tmpDir . '/' . $im16, $avatarDir . '/a16.png');
        unlink($tmpDir . '/' . str_replace('.png', '', $im48));
        unlink($tmpDir . '/' . str_replace('.png', '', $im16));
    }

    public function deleteAvatarEvent($runData) {
        $userId = $runData->getUserId();
        $avatarDir = WIKIDOT_ROOT . '/web/files--common/images/avatars/';
        $avatarDir .= '' . floor($userId / 1000) . '/' . $userId;
        unlink($avatarDir . '/a48.png');
        unlink($avatarDir . '/a16.png');
    }

    public function uploadAvatarUriEvent($runData) {
        $pl = $runData->getParameterList();
        $uri = $pl->getParameterValue("uri");
        
        if (preg_match("/^(http[s]?:\/\/)|(ftp:\/\/)[a-zA-Z0-9\-]+\/.*/", $uri) == 0) {
            $runData->ajaxResponseAdd("status", "wrong_uri");
            return;
        }
        
        $fileContent = file_get_contents($uri);
        if (!$fileContent) {
            $runData->ajaxResponseAdd("status", "fetch_failed");
            return;
        }
        $dir = WIKIDOT_ROOT . '/web/files--common/tmp/avatars-upload';
        $tmpname = tempnam($dir, "uriup");
        
        file_put_contents($tmpname, $fileContent);
        
        $fmime = FileMime::mime($tmpname);

        if ($fmime != "image/png" && $fmime != "image/jpeg" && $fmime != "image/gif") {
            $status = "wrong_mime";
            $runData->ajaxResponseAdd("status", $status);
            $runData->ajaxResponseAdd("mime", $fmime);
            return;
        }
        
        $size = getimagesize($tmpname);
        if ($size == false) {
            $status = "not_image";
            $runData->ajaxResponseAdd("status", $status);
            return;
        }
        if ($size[0] < 16 || $size[1] < 16) {
            $status = "too_small";
            $runData->contextAdd("status", $status);
            return;
        }
        
        $im48fn = tempnam($dir, "av") . ".png";
        $im16fn = tempnam($dir, "av") . ".png";
        
        if ($size[0] != 100 && $size[1] != 100) {
            // need to resize...	
            $w = $size[0];
            $h = $size[1];
            $r = $h / $w;
            $cmd = "convert -resize '100x100>' " . escapeshellarg($tmpname) . " " . escapeshellarg($im48fn);
            
            exec($cmd, $out);
            
            $runData->contextAdd("originalSize", $size);
            $runData->contextAdd("resized", true);
        } else {
            $cmd = "convert  " . escapeshellarg($tmpname) . " " . escapeshellarg($im48fn);
            exec($cmd);
        }
        $cmd = "convert -resize 16x16! -unsharp 0x1.0+1.0+0.10 " . escapeshellarg($im48fn) . " " . escapeshellarg($im16fn);
        exec($cmd);
        
        $runData->ajaxResponseAdd("im48", basename($im48fn));
        $runData->ajaxResponseAdd("im16", basename($im16fn));

    }

    public function saveAboutEvent($runData) {
        $pl = $runData->getParameterList();
        $userId = $runData->getUserId();
        $profile = DB_ProfilePeer::instance()->selectByPrimaryKey($userId);
        
        // now manually get all files...
        $realName = $pl->getParameterValue("real_name");
        $gender = $pl->getParameterValue("gender");
        $birthdayDay = $pl->getParameterValue("birthday_day");
        $birthdayMonth = $pl->getParameterValue("birthday_month");
        $birthdayYear = $pl->getParameterValue("birthday_year");
        
        $about = $pl->getParameterValue("about");
        $website = $pl->getParameterValue("website");
        $imAim = $pl->getParameterValue("im_aim");
        $imGaduGadu = $pl->getParameterValue("im_gadu_gadu");
        $imGoogleTalk = $pl->getParameterValue("im_google_talk");
        $imIcq = $pl->getParameterValue("im_icq");
        $imJabber = $pl->getParameterValue("im_jabber");
        $imMsn = $pl->getParameterValue("im_msn");
        $imYahoo = $pl->getParameterValue("im_yahoo");
        
        $location = $pl->getParameterValue("location");
        
        $profile->setRealName($realName);
        $profile->setGender($gender);
        
        // check date
        $d = getdate();
        if (checkdate((int) $birthdayMonth, (int) $birthdayDay, (int) $birthdayYear) && $birthdayYear < $d['year']) {
            $profile->setBirthdayDay($birthdayDay);
            $profile->setBirthdayMonth($birthdayMonth);
            $profile->setBirthdayYear($birthdayYear);
        }
        
        $profile->setAbout(substr($about, 0, 220));
        
        if (preg_match("/^(http[s]?:\/\/)|(ftp:\/\/)[a-zA-Z0-9\-]+\/.*/", $website) !== 0) {
            $profile->setWebsite($website);
        }
        
        $profile->setImAim($imAim);
        $profile->setImGaduGadu($imGaduGadu);
        $profile->setImGoogleTalk($imGoogleTalk);
        $profile->setImIcq($imIcq);
        $profile->setImJabber($imJabber);
        $profile->setImMsn($imMsn);
        $profile->setImYahoo($imYahoo);
        
        $profile->setLocation($location);
        
        $profile->save();
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    
    }

    public function changeScreenNameEvent($runData) {
        $user = $runData->getUser();
        $userId = $user->getUserId();
        $profile = $user->getProfile();
        
        if ($profile->getChangeScreenNameCount() >= 2) {
            throw new ProcessException('Your are allowed to change your screen name only 2 times.');
        }
        
        $pl = $runData->getParameterList();
        $name = trim($pl->getParameterValue("screenName"));
        
        if ($name == $user->getNickName()) {
            throw new ProcessException("Your new and current screen names are the same.");
        }
        $db = Database::connection();
        $db->begin();
        
        $unixified = WDStringUtils::toUnixName($name);
        if (strlen($name) < 2) {
            throw new ProcessException(_("You really should provide the screen name you want to use."));
        }
        if (strlen8($name) > 20) {
            throw new ProcessException(_("Your screen name should not be longer than 20 characters."));
        }
        if (preg_match('/^[ _a-zA-Z0-9-\!#\$%\^\*\(\)]+$/', $name) == 0) {
            throw new ProcessException(_("Only alphanumeric characters (+a few special) can be used in the screen name."));
        }
        if (strlen($unixified) < 2) {
            throw new ProcessException(_("It seems there are too less alphanumeric characters in your screen name"));
        }

        //handle forbidden names
        $unixName = WDStringUtils::toUnixName($name);
        
        $forbiddenUnixNames = explode("\n", file_get_contents(WIKIDOT_ROOT . '/conf/forbidden_user_names.conf'));
        foreach ($forbiddenUnixNames as $f) {
            if (preg_match($f, $unixName) > 0) {
                throw new ProcessException(_('For some reason this name is not allowed or is reserved for future use.'));
            }
        }
        
        // check if user does not exist
        $c = new Criteria();
        $c->add("unix_name", $unixified);
        $u = DB_OzoneUserPeer::instance()->selectOne($c);
        if ($u != null) {
            throw new ProcessException(_("A user with this screen name (or very similar) already exists."));
        }
        
        // rename the profile page
        $c = new Criteria();
        $c->add("unix_name", "profiles");
        $nsite = DB_SitePeer::instance()->selectOne($c);
        
        $pageName = 'profile:' . $user->getUnixName();
        
        $c = new Criteria();
        $c->add('site_id', $nsite->getSiteId());
        $c->add('unix_name', $pageName);
        
        $page = DB_PagePeer::instance()->selectOne($c);
        if (!$page) {
            throw new ProcessException('Internal error');
        }
        $metadata = $page->getMetadata();
        $metadata->setUnixName('profile:' . $unixified);
        $page->setUnixName('profile:' . $unixified);
        $metadata->save();
        $page->save();
        // outdate page cache
        $outdater = new Outdater();
        $outdater->pageEvent("rename", $page, $pageName);
        // now, try to apply new name!!!

        $user->setNickName($name);
        $user->setUnixName($unixified);
        $user->save();
        
        $profile->setChangeScreenNameCount($profile->getChangeScreenNameCount() + 1);
        $profile->save();
        
        $db->commit();
    }

}
