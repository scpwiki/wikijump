<?php

namespace Wikidot\Actions;

use Ozone\Framework\SmartyAction;
use Wikidot\Utils\GlobalProperties;
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
        header("HTTP/1.1 301 Moved Permanently");
        header("Location: " . route('profile.show'));
    }

    public function setAvatarEvent($runData)
    {
        header("HTTP/1.1 301 Moved Permanently");
        header("Location: " . route('profile.show'));
    }

    public function deleteAvatarEvent($runData)
    {
        header("HTTP/1.1 301 Moved Permanently");
        header("Location: " . route('profile.show'));
    }

    public function uploadAvatarUriEvent($runData)
    {
        header("HTTP/1.1 301 Moved Permanently");
        header("Location: " . route('profile.show'));
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
