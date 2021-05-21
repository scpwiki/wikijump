<?php

namespace Wikidot\Actions;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\SmartyAction;
use Wikidot\Config\ForbiddenNames;

use Wikidot\DB\SitePeer;
use Wikidot\DB\PagePeer;
use Wikidot\Utils\FileMime;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\Outdater;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;
use Wikidot\Utils\WDStringUtils;
use Wikijump\Models\User;

class AccountProfileAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        $userId = $runData->getUserId();
        if(!$userId) {
            throw new WDPermissionException(_("Not allowed. You should login first."));
        }
        return true;
    }

    public function perform($r)
    {
    }

    public function uploadAvatarEvent($runData)
    {
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
        $dir = WIKIJUMP_ROOT . '/web/files--common/tmp/avatars-upload';

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

    public function setAvatarEvent($runData)
    {

        $userId = $runData->getUserId();

        $pl = $runData->getParameterList();
        $im48 = $pl->getParameterValue("im48");
        $im16 = $pl->getParameterValue("im16");

        $avatarDir = WIKIJUMP_ROOT . '/web/files--common/images/avatars/';
        $avatarDir .= '' . floor($userId / 1000) . '/' . $userId;

        mkdirfull($avatarDir);
        $tmpDir = WIKIJUMP_ROOT . '/web/files--common/tmp/avatars-upload';
        rename($tmpDir . '/' . $im48, $avatarDir . '/a48.png');
        rename($tmpDir . '/' . $im16, $avatarDir . '/a16.png');
        unlink($tmpDir . '/' . str_replace('.png', '', $im48));
        unlink($tmpDir . '/' . str_replace('.png', '', $im16));
    }

    public function deleteAvatarEvent($runData)
    {
        $userId = $runData->getUserId();
        $avatarDir = WIKIJUMP_ROOT . '/web/files--common/images/avatars/';
        $avatarDir .= '' . floor($userId / 1000) . '/' . $userId;
        unlink($avatarDir . '/a48.png');
        unlink($avatarDir . '/a16.png');
    }

    public function uploadAvatarUriEvent($runData)
    {
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
        $dir = WIKIJUMP_ROOT . '/web/files--common/tmp/avatars-upload';
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

    public function saveAboutEvent($runData)
    {
        $pl = $runData->getParameterList();
        $userId = $runData->getUserId();
        $profile = User::find($userId);

        // now manually get all files...
        $realName = $pl->getParameterValue("real_name");
        $pronouns = $pl->getParameterValue("pronouns");
        $birthdayDay = $pl->getParameterValue("birthday_day");
        $birthdayMonth = $pl->getParameterValue("birthday_month");
        $birthdayYear = $pl->getParameterValue("birthday_year");

        $about = $pl->getParameterValue("about");
        $website = $pl->getParameterValue("website");
        $imIcq = $pl->getParameterValue("im_icq");
        $imJabber = $pl->getParameterValue("im_jabber");

        $location = $pl->getParameterValue("location");

        $profile->real_name = $realName;
        $profile->pronouns = $pronouns;

        // check date
        $d = getdate();
        if (checkdate((int) $birthdayMonth, (int) $birthdayDay, (int) $birthdayYear) && $birthdayYear < $d['year']) {
            // Pad with zeroes if needed.
            if($birthdayMonth < 10) { $birthdayMonth = "0".$birthdayMonth; }
            if($birthdayDay < 10) { $birthdayDay = "0".$birthdayDay; }

            $profile->dob = "$birthdayYear-$birthdayMonth-$birthdayDay";
        }

        $profile->bio = substr($about, 0, 220);

        if (preg_match("/^(http[s]?:\/\/)|(ftp:\/\/)[a-zA-Z0-9\-]+\/.*/", $website) !== 0) {
            $profile->about_page = $website;
        }

        $profile->save();
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function changeScreenNameEvent($runData)
    {
        $user = $runData->getUser();

        if ($user->username_changes >= config('wikijump.username_change_limit')) {
            throw new ProcessException(__('Maximum username changes allowed: '.config('wikijump.username_change_limit')));
        }

        $pl = $runData->getParameterList();
        $name = trim($pl->getParameterValue('screenName'));

        if ($name == $user->username) {
            throw new ProcessException(__('Your current and new usernames are the same.'));
        }

        $unixified = WDStringUtils::toUnixName($name);
        if (strlen($name) < config('wikijump.username_min')) {
            throw new ProcessException(__('Minimum characters for a username:').config('wikijump.username_min'));
        }
        if (strlen($name) > config('wikijump.username_max')) {
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

        foreach (config('wikijump.forbidden_usernames') as $regex) {
            if (preg_match($regex, $unixName) > 0) {
                throw new ProcessException(__('Account creation failed: Username is blocked from registration.'));
            }
        }

        // check if user does not exist
        $u = User::where('unix_name', $unixified)->first();
        if ($u != null) {
            throw new ProcessException(__("A user with this screen name (or very similar) already exists."));
        }

        $user->username = $name;
        $user->unix_name = $unixified;
        $user->username_changes++;
        $user->save();
    }
}
